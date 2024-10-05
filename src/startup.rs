use actix_web::dev::Server;
use actix_web::web::{get, post, Data};
use actix_web::{App, HttpServer};
use sqlx::PgPool;
use std::io::Error;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger; // Transmission Control Protocol: [TCP]

use crate::configuration::Settings;
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};

// NOTE: HTTP & TCP is a protocol

pub struct Application;

impl Application {
    pub async fn build(config: Settings) -> Result<Server, Error> {
        let connection_pool = PgPool::connect_lazy_with(config.database.connect_options());

        // Build an `EmailClient` using `configuration`
        let sender_email = config
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let timeout = config.email_client.timeout();

        let email_client = EmailClient::new(
            config.email_client.base_url,
            sender_email,
            config.email_client.authorization_token,
            timeout,
        );

        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address)?;

        Self::run(listener, connection_pool, email_client)
    }

    fn run(
        listener: TcpListener,
        db_pool: PgPool,
        email_client: EmailClient,
    ) -> Result<Server, std::io::Error> {
        let db_pool = Data::new(db_pool);
        let email_client = Data::new(email_client);

        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .route("/health-check", get().to(health_check))
                .route("/subscriptions", post().to(subscribe))
                .app_data(db_pool.clone())
                .app_data(email_client.clone())
        })
        .listen(listener)?
        .run();

        Ok(server)
    }
}
