use anchor::prelude::{Cluster, DockerClient, Manifest};
use std::error::Error;

const MANIFEST_FILEPATH: &str = "./input/manifest.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manifest = Manifest::load(MANIFEST_FILEPATH).expect("Failed to read manifest file.");
    let credentials = get_ecr_credentials().await?;
    let client = DockerClient::new(credentials).await?;

    let mut cluster = Cluster::new(&client, manifest).await?;
    cluster.start(|status| println!("{:?}", status)).await?;
    println!("All containers are ready.");

    Ok(())
}
