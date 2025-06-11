use anchor::prelude::DockerClient;
use std::error::Error;

mod auth;
use auth::get_ecr_credentials;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = DockerClient::new(credentials).await?;

    for container in client.list_containers().await? {
        if let Some(names) = container.names {
            if !names.is_empty() {
                println!("- {}", names.join(", "));
            } else {
                println!("- Unnamed container");
            }
        } else {
            println!("- Unnamed container");
        }
    }

    Ok(())
}
