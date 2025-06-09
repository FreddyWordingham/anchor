use anchor::prelude::{Manifest, ManifestError};

const MANIFEST_FILEPATH: &str = "./assets/manifest.json";

#[tokio::main]
async fn main() -> Result<(), ManifestError> {
    let manifest = Manifest::load(MANIFEST_FILEPATH).expect("Failed to read manifest file.");
    for (name, container) in manifest.containers() {
        println!("{}:", name);
        println!(" - {}", container.uri);
        println!(" - {}", container.command);
        println!(
            " - {}",
            container
                .port_mappings
                .iter()
                .map(|(port, host_port)| format!("{}:{}", port, host_port))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(())
}
