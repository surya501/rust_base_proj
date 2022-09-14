use base_proj::telemetry::{create_subscriber, init_subscriber};
use base_proj::{configuration::get_configuration, startup::run};
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // configuration for the tracing capabilities
    let subscriber = create_subscriber("base_proj".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind random port");

    let connection_string = configuration.database.connection_string();
    let connection = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    // let's bubble up the error from the run function if the bind fails
    run(listener, connection)?.await
}
