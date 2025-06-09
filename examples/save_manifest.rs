use anchor::prelude::*;

const MANIFEST_FILEPATH: &str = "./output/manifest.json";

#[tokio::main]
async fn main() -> Result<(), ManifestError> {
    // Create a new empty manifest to which we will add containers.
    let mut manifest = Manifest::empty();

    // Add a web container with Nginx
    manifest.add_container(
        "web".to_string(),
        Container {
            uri: "docker.io/library/nginx:latest".to_string(),
            command: Command::Run,
            port_mappings: vec![(80, 8080)],
        },
    )?;

    // And also add a database container with Postgres.
    manifest.add_container(
        "db".to_string(),
        Container {
            uri: "docker.io/library/postgres:latest".to_string(),
            command: Command::Run,
            port_mappings: vec![(5432, 5432)],
        },
    )?;

    // Save the manifest to a JSON configuration file.
    manifest.save(MANIFEST_FILEPATH).expect("Failed to save manifest file.");

    Ok(())
}
