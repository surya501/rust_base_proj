use base_proj::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // let's bubble up the error from the run function if the bind fails
    run()?.await
}
