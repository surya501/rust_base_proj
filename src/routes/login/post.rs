use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;
pub async fn login_form() -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, "/"))
        .finish()
}
