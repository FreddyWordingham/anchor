use anchor::prelude::DockerClient;
use std::error::Error;

mod auth;
use auth::get_ecr_credentials;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = DockerClient::new(credentials).await?;

    for image in client.list_images().await? {
        println!("- {}", image.repo_tags.join(", "));
    }

    Ok(())
}
