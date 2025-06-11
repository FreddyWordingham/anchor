use anchor::prelude::{DockerClient, Manifest, Server, ServerStatus};
use std::error::Error;

mod auth;
use auth::get_ecr_credentials;

const MANIFEST_FILEPATH: &str = "./input/manifest.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manifest = Manifest::load(MANIFEST_FILEPATH).expect("Failed to read manifest file.");
    let credentials = get_ecr_credentials().await?;
    let client = DockerClient::new(credentials).await?;

    let mut server = Server::new(&client, manifest).await?;
    loop {
        let status = server.next().await?;
        if status == ServerStatus::Ready {
            break;
        }
        println!("{:?}", status);
    }
    println!("All containers are ready.");

    Ok(())
}
