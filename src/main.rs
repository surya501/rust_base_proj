use std::net::TcpListener;

use base_proj::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind random port");
    // let's bubble up the error from the run function if the bind fails
    run(listener)?.await
}
