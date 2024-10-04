use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::Configuration;
use zero2prod::email_client::EmailClient;
use zero2prod::startup;
use zero2prod::telemetry::Telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = Configuration::get().expect("Failed to read configuration.");

    Telemetry::init_subscriber(config.application.name, "info".into(), std::io::stdout);

    let address = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(address)?;

    let connection_pool = PgPool::connect_lazy_with(config.database.connect_options());

    // Build an `EmailClient` using `configuration`
    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = EmailClient::new(config.email_client.base_url, sender_email);

    startup::run(listener, connection_pool, email_client)?.await?;

    Ok(())
}
