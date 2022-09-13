use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

// subscribe handler
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let uuid = Uuid::new_v4();

    log::info!(
        "requestId: {} - New subsciber addition: {:?} <{}>",
        uuid,
        form.name,
        form.email
    );
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await
    {
        Ok(_) => {
            log::info!(
                "requestId: {} - New subscription: {} <{}>",
                uuid,
                &form.name,
                &form.email
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            log::error!("requestId: {} - Failed to execute query: {:?}", uuid, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
