use anchor::prelude::{DockerClient, get_ecr_credentials};
use std::error::Error;

const IMAGE_REFERENCE: &str = "939027885851.dkr.ecr.eu-west-2.amazonaws.com/uncertainty-engine-add-node:latest";
const CONTAINER_NAME: &str = "add-node";
const PORT_MAPPINGS: [(u16, u16); 1] = [(8000, 8001)];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = DockerClient::new(credentials).await?;

    if client.is_container_built(CONTAINER_NAME).await? {
        println!("Container {} is already built.", CONTAINER_NAME);
    } else {
        println!("Building container {}...", CONTAINER_NAME);
        client
            .build_container(IMAGE_REFERENCE, CONTAINER_NAME, &PORT_MAPPINGS)
            .await?;
    }

    Ok(())
}
