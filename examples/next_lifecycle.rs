use anchor::prelude::{Client, MountType, ResourceStatus, get_ecr_credentials};
use std::error::Error;

const IMAGE_REF: &str = "939027885851.dkr.ecr.eu-west-2.amazonaws.com/uncertainty-engine-add-node:latest";
const CONTAINER_NAME: &str = "node-add";
const PORT_MAPPINGS: &[(u16, u16)] = &[(8000, 8000)];
const ENV_VARS: &[(&str, &str)] = &[];
const MOUNTS: &[MountType] = &[];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let credentials = get_ecr_credentials().await?;
    let client = Client::new(credentials).await?;

    let status = client.get_resource_status(IMAGE_REF, CONTAINER_NAME).await?;
    match status {
        ResourceStatus::Missing => {
            println!("Pulling image...");
            client.pull_image(IMAGE_REF).await?;
        }
        ResourceStatus::Available => {
            println!("Building container...");
            client
                .build_container(IMAGE_REF, CONTAINER_NAME, PORT_MAPPINGS, ENV_VARS, MOUNTS)
                .await?;
        }
        ResourceStatus::Built => {
            println!("Starting container...");
            client.start_container(CONTAINER_NAME).await?;
        }
        ResourceStatus::Running => {
            println!("Container is already running.");
        }
    }

    Ok(())
}
