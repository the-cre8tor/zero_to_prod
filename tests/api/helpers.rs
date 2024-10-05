//! tests/api/helpers.rs

use redact::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::LazyLock;
use uuid::Uuid;
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

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn spawn_app() -> TestApp {
        // The first time `initialize` is invoked the code in `TRACING` is executed.
        // All other invocations will instead skip execution.
        LazyLock::force(&TRACING);

        // Randomise configuration to ensure test isolation
        let configuration = {
            let mut config = Configuration::get().expect("Failed to read configuration.");
            // We randomly create new database name for test purposes
            config.database.database_name = Uuid::new_v4().to_string();
            config.application.port = 0;

            config
        };

        // Create and migrate the database
        let connection_pool = TestApp::configure_database(&configuration.database).await;

        // Launch the application as a background task
        let application = Application::build(&configuration, connection_pool.clone())
            .await
            .expect("Failed to build application");

        let address = format!("http://127.0.0.1:{}", application.port());
        let _ = tokio::spawn(application.run_until_stopped());

        Self {
            address,
            db_pool: connection_pool,
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
}
