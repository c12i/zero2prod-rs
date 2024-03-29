use std::net::TcpListener;

use actix_session::storage::RedisSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::{dev::Server, web, App, HttpServer};
use actix_web_flash_messages::storage::CookieMessageStore;
use actix_web_flash_messages::FlashMessagesFramework;
use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{
    admin_dashboard, change_password, change_password_form, confirm, health_check, home, login,
    login_form, publish_newsletter, subscribe,
};

pub struct Application {
    port: u16,
    server: Server,
}

pub struct ApplicationBaseUrl(pub String);

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        // Build postgres connection pool
        let db_connection_pool = Application::get_connection_pool(&configuration.database)
            .await
            .expect("Failed to connect to Postgres");
        // Build an `EmailClient`
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration
                .email_client
                .authorization_token
                .expose_secret()
                .to_string(),
            timeout,
        );
        // Build `TcpListener`
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        // Get the bound port
        let port = listener.local_addr().unwrap().port();
        let server = Application::run(
            listener,
            db_connection_pool,
            email_client,
            configuration.application.base_url,
            HmacSecret(configuration.application.hmac_secret),
            configuration.redis_uri,
        )
        .await?;
        // Save the bound port in the `Application` fields
        Ok(Self { port, server })
    }

    pub async fn get_connection_pool(
        configuration: &DatabaseSettings,
    ) -> Result<PgPool, sqlx::Error> {
        PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_with(configuration.with_db())
            .await
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    async fn run(
        listener: TcpListener,
        db_connection_pool: PgPool,
        email_client: EmailClient,
        base_url: String,
        hmac_secret: HmacSecret,
        redis_uri: Secret<String>,
    ) -> Result<Server, anyhow::Error> {
        let db_connection_pool = web::Data::new(db_connection_pool);
        let email_client = web::Data::new(email_client);
        let base_url = web::Data::new(ApplicationBaseUrl(base_url));
        let hmac_secret = web::Data::new(hmac_secret);
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
                .route("/healthz", web::get().to(health_check))
                .route("/subscriptions", web::post().to(subscribe))
                .route("/subscriptions/confirm", web::get().to(confirm))
                .route("/newsletters", web::post().to(publish_newsletter))
                .route("/login", web::get().to(login_form))
                .route("/login", web::post().to(login))
                .route("/", web::get().to(home))
                .route("/admin/dashboard", web::get().to(admin_dashboard))
                .route("/admin/password", web::get().to(change_password_form))
                .route("/admin/password", web::post().to(change_password))
                .app_data(db_connection_pool.clone())
                .app_data(email_client.clone())
                .app_data(base_url.clone())
                .app_data(hmac_secret.clone())
        })
        .listen(listener)?
        .run();
        Ok(server)
    }

    // A more expressive name that makes it clear that this function only returns when
    // the application is stopped
    pub async fn run_server_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

// Using Secret<String> as the type injected into the application state is far from ideal.
// String is a primitive type and there is a significant risk of conflict - i.e.
// another middleware or service registering another Secret<String>
// against the application state, overriding our HMAC secret (or vice versa).
#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);
