use anchor::prelude::{DockerClient, get_ecr_credentials};
use std::error::Error;

const CONTAINER_NAME: &str = "add-node";
const IMAGE_REFERENCE: &str = "939027885851.dkr.ecr.eu-west-2.amazonaws.com/uncertainty-engine-add-node:latest";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = DockerClient::new(credentials).await?;

    if client.is_container_running(CONTAINER_NAME).await? {
        println!("Running");
    } else if client.is_container_built(CONTAINER_NAME).await? {
        println!("Built");
    } else if client.is_image_downloaded(IMAGE_REFERENCE).await? {
        println!("Downloaded");
    } else {
        println!("Non existent");
    }

    Ok(())
}
