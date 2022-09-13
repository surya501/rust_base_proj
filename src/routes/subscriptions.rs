use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

// subscribe handler
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let uuid = Uuid::new_v4();

    let request_span = tracing::info_span!("subscribe", request_id = %uuid, email = %form.email, name = %form.name);

    let _req_span_entered = request_span.enter();

    let query_span = tracing::info_span!("inserting_subscriber in the database");
    tracing::info!(
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
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "requestId: {} - New subscription: {} <{}>",
                uuid,
                &form.name,
                &form.email
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("requestId: {} - Failed to execute query: {:?}", uuid, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
