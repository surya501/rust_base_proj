use env_logger::Env;
use sqlx::PgPool;
use std::net::TcpListener;

use base_proj::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Before we do anything, let's setup the logger;
    // We'll use the env_logger crate, which is a popular logging framework
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

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
