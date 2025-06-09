use serde::{Deserialize, Serialize};

use crate::command::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct Container {
    pub uri: String,
    pub port_mappings: Vec<(u16, u16)>,
    pub command: Command,
}
