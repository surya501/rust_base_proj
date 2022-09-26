use base_proj::email_client::EmailClient;
use base_proj::telemetry::{create_subscriber, init_subscriber};
use base_proj::{configuration::get_configuration, startup::run};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // configuration for the tracing capabilities
    let subscriber = create_subscriber("base_proj".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address).expect("Failed to bind random port");

    let connection = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = EmailClient::new(
        sender_email,
        configuration.email_client.base_url,
        configuration.email_client.authorization_token,
    );

    // let's bubble up the error from the run function if the bind fails
    run(listener, connection, email_client)?.await
}
