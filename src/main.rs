use sqlx::PgPool;
use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use base_proj::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // configuration for the tracing capabilities

    // First capture all the logs to the tracing.
    LogTracer::init().expect("Failed to set logger.");
    // Check for RUST_LOG env variable and if not set use the info as default.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    // Let's send the output to stdout; later change this.
    let formatting_layer = BunyanFormattingLayer::new("base_proj".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");

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
