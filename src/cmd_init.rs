use std::path::PathBuf;

use utility::*;
use data_structs::*;
use commands_helper::{Action};

pub fn usage_message() -> &'static str {
    return "-init\n\t· Initialize tiny rust project";
}

pub fn run_command(cwd: PathBuf, args: Vec<String>) {
    let action = Action::new()
}
