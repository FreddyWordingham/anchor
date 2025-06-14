use anchor::prelude::{DockerClient, get_ecr_credentials};
use std::error::Error;

const CONTAINER_NAME: &str = "add-node";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = DockerClient::new(credentials).await?;

    if !client.is_container_built(CONTAINER_NAME).await? {
        println!("Container {} does not exist.", CONTAINER_NAME);
    } else {
        println!("Deleting container {}...", CONTAINER_NAME);
        client.remove_container(CONTAINER_NAME).await?;
    }

    Ok(())
}
