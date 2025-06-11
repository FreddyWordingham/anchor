use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Ignore,
    Download,
    Build,
    Run,
}

impl Display for Command {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        match self {
            Command::Ignore => write!(fmt, "Ignore"),
            Command::Download => write!(fmt, "Download"),
            Command::Build => write!(fmt, "Build"),
            Command::Run => write!(fmt, "Run"),
        }
    }
}
