use anchor::prelude::{DockerClient, get_ecr_credentials};
use std::error::Error;

const IMAGE_REFERENCE: &str = "939027885851.dkr.ecr.eu-west-2.amazonaws.com/uncertainty-engine-add-node:latest";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = DockerClient::new(credentials).await?;

    if !client.is_image_downloaded(IMAGE_REFERENCE).await? {
        println!("Image {} does not exist.", IMAGE_REFERENCE);
    } else {
        println!("Deleting image {}...", IMAGE_REFERENCE);
        client.remove_image(IMAGE_REFERENCE).await?;
    }

    Ok(())
}
