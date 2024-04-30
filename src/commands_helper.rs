use std::process::{Command, ExitStatus};
use std::path::PathBuf;
use std::fs;
use std::ffi::OsStr;

use utility::{info, error, path_move};

pub struct Action {
    cmd: Command,
}

impl Action {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Action {
        return Action {
            cmd: Command::new(program)
        };
    }

    pub fn run_new(cmd_creator: impl Fn() -> Action) -> std::io::Result<ExitStatus> {
        let mut action = cmd_creator();
        return action.run();
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Action {
        self.cmd.arg(arg);
        self
    }

    pub fn run(&mut self) -> std::io::Result<ExitStatus> {
        return self.cmd.status();
    }

    pub fn run_nofail(&mut self) -> ExitStatus {
        return self.run().expect("Failed to execute command");
    }
}

