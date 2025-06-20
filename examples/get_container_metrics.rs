use anchor::prelude::{Client, get_ecr_credentials};
use std::error::Error;

const CONTAINER_NAME: &str = "node-add";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = Client::new(credentials).await?;

    let metrics = client.get_container_metrics(CONTAINER_NAME).await?;
    println!("Container Metrics: {metrics}");

    Ok(())
}
