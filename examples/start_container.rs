use anchor::prelude::{DockerClient, get_ecr_credentials};
use std::error::Error;

const CONTAINER_NAME: &str = "add-node";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = DockerClient::new(credentials).await?;

    if client.is_container_running(CONTAINER_NAME).await? {
        println!("Container {} is already running.", CONTAINER_NAME);
    } else {
        println!("Starting container {}...", CONTAINER_NAME);
        client.start_container(CONTAINER_NAME).await?;
    }

    Ok(())
}
