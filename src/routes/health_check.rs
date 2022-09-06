use actix_web::HttpResponse;

// health check handler
// check this using the following command
// curl -v http://127.0.0.1:8080/health_check
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}
