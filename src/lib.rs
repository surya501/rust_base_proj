use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::net::TcpListener;

// health check handler
// check this using the following command
// curl -v http://127.0.0.1:8080/health_check
async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(serde::Deserialize)]
struct FormData {
    name: String,
    email: String,
}

// subscribe handler
async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    // look no await here
    Ok(server)
}
