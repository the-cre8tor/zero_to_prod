use zero_to_prod::configuration::Configuration;
use zero_to_prod::startup::Application;
use zero_to_prod::telemetry::Telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = Configuration::get().expect("Failed to read configuration.");

    Telemetry::init_subscriber(&config.application.name, "info".into(), std::io::stdout);

    let connection_pool = Application::db_connection_pool(&config.database);

    let application = Application::build(config, connection_pool).await?;
    application.run_until_stopped().await?;

    Ok(())
}
