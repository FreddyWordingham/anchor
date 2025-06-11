use anchor::prelude::{DockerClient, get_ecr_credentials};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = DockerClient::new(credentials).await?;

    if client.is_docker_running().await {
        println!("Docker is running.");
    } else {
        println!("Docker is NOT running.");
    }

    Ok(())
}
