use zero_to_prod::configuration::Configuration;
use zero_to_prod::startup::Application;
use zero_to_prod::telemetry::Telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = Configuration::get().expect("Failed to read configuration.");

    Telemetry::init_subscriber(&config.application.name, "info".into(), std::io::stdout);

    let server = Application::build(config).await?;
    server.await?;

    Ok(())
}
