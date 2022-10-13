use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::email_client::EmailClient;
use crate::routes::{confirm, health, home, login_form, publish_newsletter, subscribe};

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let email_client = Data::new(email_client);
    let base_url = Data::new(base_url);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            .route("/login", web::post().to(login_form))
            .route("/", web::get().to(home))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    // look no await here
    Ok(server)
}
