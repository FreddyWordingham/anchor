use anchor::prelude::start_docker_daemon;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    start_docker_daemon()?;
    println!("Docker daemon launched successfully.");

    Ok(())
}
