use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Ignore,
    Download,
    Build,
    Run,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Ignore => write!(f, "Ignore"),
            Command::Download => write!(f, "Download"),
            Command::Build => write!(f, "Build"),
            Command::Run => write!(f, "Run"),
        }
    }
}
