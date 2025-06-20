use anchor::prelude::{Client, get_ecr_credentials};
use std::error::Error;

const IMAGE_REF: &str = "939027885851.dkr.ecr.eu-west-2.amazonaws.com/uncertainty-engine-add-node:latest";
const CONTAINER_NAME: &str = "node-add";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = Client::new(credentials).await?;

    let status = client.get_resource_status(IMAGE_REF, CONTAINER_NAME).await?;
    println!("Resource Status: {status}");

    Ok(())
}
