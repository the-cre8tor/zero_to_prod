//! tests/api/helpers.rs

use redact::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::{net::TcpListener, sync::LazyLock};
use uuid::Uuid;
use zero_to_prod::{
    configuration::{Configuration, DatabaseSettings},
    email_client::EmailClient,
    startup,
    telemetry::Telemetry,
};

// Ensure that the `tracing` stack is only initialised once using `LazyLock`
static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    let use_test_log = std::env::var("TEST_LOG").is_ok();

    if use_test_log {
        Telemetry::init_subscriber(subscriber_name, default_filter_level, std::io::stdout);
    } else {
        Telemetry::init_subscriber(subscriber_name, default_filter_level, std::io::sink);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn spawn_app() -> TestApp {
        // The first time `initialize` is invoked the code in `TRACING` is executed.
        // All other invocations will instead skip execution.
        LazyLock::force(&TRACING);

        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind radom port");
        let socket_address = listener.local_addr().unwrap();
        let port = socket_address.port();

        let address = format!("http://127.0.0.1:{}", port);

        let mut configuration = Configuration::get().expect("Failed to read configuration.");

        // We randomly create new database name for test purposes
        configuration.database.database_name = Uuid::new_v4().to_string();

        let connection_pool = TestApp::configure_database(&configuration.database).await;

        // Build a new email client
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let timeout = configuration.email_client.timeout();

        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        let server = startup::run(listener, connection_pool.clone(), email_client)
            .expect("Failed to bind address");

        let _ = tokio::spawn(server);

        Self {
            address,
            db_pool: connection_pool,
        }
    }

    pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
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
}
