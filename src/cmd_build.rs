use std::process::Command;
use std::path::PathBuf;
use data_structs::*;
use utility::{info, error};

type CmdResult = Result<(), String>;
const COMMAND_NAME: &'static str = "build";

pub fn check_args(argv: &Vec<String>) -> bool {
    if argv.len() < 1 {
        return false;
    }

    let cmd_name = COMMAND_NAME.to_string();
    if argv[0] == cmd_name {
        return true;
    }
    if argv[0] == format!("{}-run", COMMAND_NAME) {
        return true;
    }

    return false;
}

pub fn usage_message() -> (String, &'static str) {
    (
	format!("{} [-r] [dir] [-- [run-args]]", COMMAND_NAME),
	"Germinate! Compile rust based on kinoko.🍄\n\t\t\t\t Pass `-r` flag to run build after compilation.\n\t\t\t\t Any arguments passed after `--` will be passed onto the ran build"
    )
}

pub fn run_command(cwd: PathBuf, mut args: Vec<String>) -> CmdResult {
    // Remove command name
    args.remove(0);
    let run_build = if args.len() > 0 {
	let mut run_flag = false;
	for i in 0..args.len() {
	    if args[i] == "-r" {
		args.remove(i);
		run_flag = true;
		break;
	    }
	}
	run_flag
    } else {
	false
    };

    let mut build_args = Vec::new();
    let mut run_args = Vec::new();
    let mut pass_to_run = false;
    for arg in args.into_iter() {
	if pass_to_run {
	    run_args.push(arg);
	    continue;
	}
	if arg == "--" && ! pass_to_run {
	    pass_to_run = true;
	    continue;
	}
	build_args.push(arg);
    }
    
    let kin = if build_args.len() > 0 {
	let path = PathBuf::from(&build_args[0]);
	if path.exists() && path.is_dir() {
	    build_args.remove(0);
	    Kinoko::new_with_args(path, build_args)
	} else {
	    Kinoko::new_with_args((if cfg!(windows) { ".\\" } else { "./" }).into(), build_args)
	}
    } else {
	Kinoko::new_with_args((if cfg!(windows) { ".\\" } else { "./" }).into(), build_args)
    };

    return match kin.try_germinate() {
	Err(err) => Err(format!("{}", err)),
	Ok(out) => {
	    if run_build {
		let mut command = format!("{}", out.display());
		let mut cmd = Command::new(&out);
		for arg in run_args.into_iter() {
		    command = format!("{} `{}`", command, arg);
		    cmd.arg(arg);
		}
		println!("[CMD] {}", command);
		let result = cmd.status();
		if let Ok(status) = result {
		    info!("Build output is executable.");
		    if status.success() {
			info!("{} - Exited with a success", out.display());
		    } else {
			match status.code() {
			    Some(code) => info!("{} - Exited with a failure result: {}", out.display(), code),
			    None => info!("{} - Exited abruptly: Process terminated by a signal", out.display()),
			};
		    }
		} else {
		    error!("Build output was failed to be executed: {}", out.display());
		}
	    }

	    Ok(())
	},
    };
}
