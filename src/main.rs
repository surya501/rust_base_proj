use std::net::TcpListener;

use base_proj::{configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = configuration::get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind random port");
    // let's bubble up the error from the run function if the bind fails
    run(listener)?.await
}
