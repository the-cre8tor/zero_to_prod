//! tests/api/helpers.rs

use linkify::{LinkFinder, LinkKind};
use redact::Secret;
use reqwest::{Client, Response, Url};
use serde_json::Value;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::LazyLock;
use uuid::Uuid;
use wiremock::MockServer;
use zero_to_prod::{
    configuration::{Configuration, DatabaseSettings},
    startup::Application,
    telemetry::Telemetry,
};

// Ensure that the `tracing` stack is only initialised once using `LazyLock`
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test";
    let use_test_log = std::env::var("TEST_LOG").is_ok();

    if use_test_log {
        Telemetry::init_subscriber(subscriber_name, default_filter_level, std::io::stdout);
    } else {
        Telemetry::init_subscriber(subscriber_name, default_filter_level, std::io::sink);
    }
});

pub struct ConfirmationLinks {
    pub html: Url,
    pub plain_text: Url,
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
}

impl TestApp {
    pub async fn spawn_app() -> TestApp {
        // The first time `initialize` is invoked the code in `TRACING` is executed.
        // All other invocations will instead skip execution.
        LazyLock::force(&TRACING);

        // Launch a mock server to stand in for Postmark's API
        let email_server = MockServer::start().await;

        // Randomise configuration to ensure test isolation
        let configuration = {
            let mut config = Configuration::get().expect("Failed to read configuration.");

            // We randomly create new database name for test purposes
            config.database.database_name = Uuid::new_v4().to_string();
            config.application.port = 0;
            config.email_client.base_url = email_server.uri();

            config
        };

        // Create and migrate the database
        let connection_pool = TestApp::configure_database(&configuration.database).await;

        // Launch the application as a background task
        let application = Application::build(configuration, connection_pool.clone())
            .await
            .expect("Failed to build application");
        let application_port = application.port();

        let address = format!("http://127.0.0.1:{}", application_port);
        let _ = tokio::spawn(application.run_until_stopped());

        Self {
            address,
            port: application_port,
            db_pool: connection_pool,
            email_server,
        }
    }

    async fn configure_database(config: &DatabaseSettings) -> PgPool {
        let mut maintenance_settings = DatabaseSettings {
            database_name: "postgres".to_string(),
            username: "postgres".to_string(),
            password: Secret::new("password".to_string()),
            ..config.clone()
        };

        let mut connection = PgConnection::connect_with(&maintenance_settings.connect_options())
            .await
            .expect("Failed to connect to Postgres.");

        // Create database.
        let create_query = format!(r#"CREATE DATABASE "{}"; "#, config.database_name);
        connection
            .execute(create_query.as_str())
            .await
            .expect("Failed to create database.");

        // Migrate database.
        maintenance_settings.database_name = config.database_name.clone();
        let connection_pool = PgPool::connect_with(maintenance_settings.connect_options())
            .await
            .expect("Failed to connect to Postgres.");

        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("Failed to migrate the database");

        connection_pool
    }

    pub async fn post_subscriptions(&self, body: String) -> Response {
        Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: Value = serde_json::from_slice(&email_request.body).unwrap();

        // Extract the link from one of the request fields.
        let get_link = |input: &str| {
            let links: Vec<_> = LinkFinder::new()
                .links(input)
                .filter(|value| *value.kind() == LinkKind::Url)
                .collect();

            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = Url::parse(&raw_link).unwrap();

            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }
}
