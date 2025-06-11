use anchor::prelude::DockerClient;
use std::error::Error;

mod auth;
use auth::get_ecr_credentials;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = DockerClient::new(credentials).await?;

    let platform = client.platform();
    println!("Docker platform: {}", platform);

    Ok(())
}
