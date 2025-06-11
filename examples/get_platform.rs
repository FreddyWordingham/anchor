use anchor::prelude::{DockerClient, get_ecr_credentials};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = DockerClient::new(credentials).await?;

    let platform = client.platform();
    println!("Docker platform: {}", platform);

    Ok(())
}
