use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
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
            configuration.email_client.authorization_token,
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
        let server = Application::create_server(listener, db_connection_pool, email_client)?;
        // Save the bound port in the `Application` fields
        Ok(Self { port, server })
    }

    pub async fn get_connection_pool(
        configuration: &DatabaseSettings,
    ) -> Result<PgPool, sqlx::Error> {
        PgPoolOptions::new()
            .connect_timeout(std::time::Duration::from_secs(2))
            .connect_with(configuration.with_db())
            .await
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    fn create_server(
        listener: TcpListener,
        db_connection_pool: PgPool,
        email_client: EmailClient,
    ) -> Result<Server, std::io::Error> {
        let db_connection_pool = web::Data::new(db_connection_pool);
        let email_client = web::Data::new(email_client);
        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .route("/healthz", web::get().to(health_check))
                .route("/subscriptions", web::post().to(subscribe))
                .app_data(db_connection_pool.clone())
                .app_data(email_client.clone())
                .default_service(web::route().to(|| HttpResponse::NotFound().finish()))
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
