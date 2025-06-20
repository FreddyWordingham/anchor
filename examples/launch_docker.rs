use anchor::prelude::start_docker_daemon;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Docker daemon...");
    start_docker_daemon().await?;
    println!("Docker process started successfully.");
    Ok(())
}
