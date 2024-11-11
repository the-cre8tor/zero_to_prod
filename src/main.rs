use std::fmt::{Debug, Display};

use tokio::task::JoinError;
use zero_to_prod::configuration::Configuration;
use zero_to_prod::issue_delivery_worker::run_worker_until_stopped;
use zero_to_prod::startup::Application;
use zero_to_prod::telemetry::Telemetry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Configuration::get().expect("Failed to read configuration.");

    Telemetry::init_subscriber(&config.application.name, "info".into(), std::io::stdout);

    let connection_pool =
        Application::db_connection_pool(&config.database).expect("Failed to connect to Postgres.");

    let application = Application::build(config.clone(), connection_pool).await?;
    let worker = run_worker_until_stopped(config);

    let application_task = tokio::spawn(application.run_until_stopped());
    let worker_task = tokio::spawn(worker);

    tokio::select! {
        outcome = application_task => report_exit("API", outcome),
        outcome = worker_task => report_exit("Background worker", outcome),
    }

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(error)) => {
            tracing::error!(
                error.cause_chain = ?error,
                error.message = %error,
                "{} failed",
                task_name
            )
        }
        Err(error) => {
            tracing::error!(
                error.cause_chain = ?error,
                error.message = %error,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
