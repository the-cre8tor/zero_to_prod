use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::web::{get, post, Data};
use actix_web::{App, HttpServer};
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use redact::Secret;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::io::Error;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger; // Transmission Control Protocol: [TCP]

use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{
    confirm, health_check, home, login, login_form, publish_newsletter, subscribe,
};

// NOTE: HTTP & TCP is a protocol

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);

pub struct ApplicationBaseUrl(pub String);

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(
        config: Settings,
        connection_pool: PgPool,
    ) -> Result<Application, anyhow::Error> {
        // Build an `EmailClient` using `configuration`
        let sender_email = config
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let timeout = config.email_client.timeout();

        let email_client = EmailClient::new(
            config.email_client.base_url.to_owned(),
            sender_email,
            config.email_client.authorization_token.to_owned(),
            timeout,
        );

        let address = format!("{}:{}", config.application.host, config.application.port);
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();

        let server = Self::run(
            listener,
            connection_pool,
            email_client,
            config.application.base_url,
            config.application.hmac_secret,
            config.redis_url,
        )
        .await?;

        Ok(Self { port, server })
    }

    pub fn db_connection_pool(configuration: &DatabaseSettings) -> PgPool {
        PgPoolOptions::new().connect_lazy_with(configuration.connect_options())
    }

    async fn run(
        listener: TcpListener,
        db_pool: PgPool,
        email_client: EmailClient,
        base_url: String,
        hmac_secret: Secret<String>,
        redis_uri: Secret<String>,
    ) -> Result<Server, anyhow::Error> {
        let db_pool = Data::new(db_pool);
        let email_client = Data::new(email_client);
        let base_url = Data::new(ApplicationBaseUrl(base_url));
        let hmac_secret = HmacSecret(hmac_secret);

        let secret_key = Key::from(hmac_secret.0.expose_secret().as_bytes());
        let message_store = CookieMessageStore::builder(secret_key.clone()).build();
        let message_framework = FlashMessagesFramework::builder(message_store).build();

        let redis_store = RedisSessionStore::new(redis_uri.expose_secret()).await?;

        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .wrap(message_framework.clone())
                .wrap(SessionMiddleware::new(
                    redis_store.clone(),
                    secret_key.clone(),
                ))
                .route("/", get().to(home))
                .route("/login", get().to(login_form))
                .route("/login", post().to(login))
                .route("/health-check", get().to(health_check))
                .route("/subscriptions", post().to(subscribe))
                .route("/subscriptions/confirm", get().to(confirm))
                .route("/newsletters", post().to(publish_newsletter))
                .app_data(db_pool.clone())
                .app_data(email_client.clone())
                .app_data(base_url.clone())
        })
        .listen(listener)?
        .run();

        Ok(server)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), Error> {
        self.server.await
    }
}
