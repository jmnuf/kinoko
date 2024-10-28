use std::env; 
use std::process::ExitCode;
use std::path::{Path,PathBuf};
use std::fs::{self, File, DirEntry};
use std::io::{self, Write};
use std::rc::Rc;

// Helper
mod utility;
mod data_structs;

use utility::*;
use data_structs::*;

// Commands
mod cmd_init;
mod cmd_build;

macro_rules! print_cmd_usage {
    ($cmd: expr) => {
	println!(" {:<50}--     {}", ($cmd).0, ($cmd).1)
    };
}

fn usage(program: String) {
    println!("Usage: {} <command> [Options]", program);
    // let print_cmd_usage = |(format, desc)| println!()
    let cmd_usage = cmd_init::usage_message();
    print_cmd_usage!(cmd_usage);
    // println!(" {:>20}--  {}",  cmd_usage.0, cmd_usage.1);
    let cmd_usage = cmd_build::usage_message();
    print_cmd_usage!(cmd_usage);
    
    print_cmd_usage!(("help", "Display this help message"));
    // println!(" help{<20}  Display this help message", "--");
}

fn main() -> ExitCode {
    let cwd = match env::current_dir() {
        Ok(value) => value,
        Err(err) => {
            error!("Spores failed to reach current directory: {}", err);
            return ExitCode::FAILURE;
        },
    };
    set_cwd(&cwd);
    let mut args:Vec<String> = env::args().collect();
    let program = args.remove(0);
    if args.is_empty() {
	error!("No command was provided.");
	usage(program);
	return ExitCode::FAILURE;
    }
    if args[0] == String::from("help") {
        usage(program);
        return ExitCode::SUCCESS;
    }

    if cmd_init::check_args(&args) {
        return match cmd_init::run_command(cwd, args) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                error!("Failed to initialize project: {}", e);
                ExitCode::FAILURE
            },
        }
    }
    if cmd_build::check_args(&args) {
	return match cmd_build::run_command(cwd, args) {
	    Ok(_) => ExitCode::SUCCESS,
	    Err(e) => {
		error!("Failed to build: {}", e);
		ExitCode::FAILURE
	    },
	}
    }

    error!("Unknown command passed by: {}", args[0]);
    
    
    return ExitCode::FAILURE;
}

#[derive(Debug)]
enum MainFnSearchError {
    MainNotFound(PathBuf),
    Unauthorized(io::Error),
    Inexistent(io::Error),
    GeneralIOErr(io::Error),
}
// Failed to spread roots in present directory
impl std::fmt::Display for MainFnSearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::Inexistent(err) => write!(f, "path is not a directory: {}", err),
            Self::Unauthorized(err) => write!(f, "missing permissions to access path: {}", err),
            Self::GeneralIOErr(err) => write!(f, "IOFailure: {}", err),
            Self::MainNotFound(path) => write!(f, "main function not found in path: {}", path.display()),
        }
    }
}
impl From<io::Error> for MainFnSearchError {
    fn from(io_err: io::Error) -> MainFnSearchError {
        match io_err.kind() {
            io::ErrorKind::PermissionDenied => MainFnSearchError::Unauthorized(io_err),
            io::ErrorKind::NotFound => MainFnSearchError::Inexistent(io_err),
            _ => MainFnSearchError::GeneralIOErr(io_err),
        }
    }
}

enum DirEntryAction {
    Ignore,
    ReadFile,
    ReadDir,
}

fn rust_file_dir_entry_checker(entry: &DirEntry) -> DirEntryAction {
    let entry_path = entry.path();
    if entry_path.is_dir() {
        if entry_path.ends_with("src") {
            return DirEntryAction::ReadDir;
        }
    }
    if !entry_path.is_file() {
        return DirEntryAction::Ignore;
    }
    if entry_path != entry_path.with_extension("rs") {
        return DirEntryAction::Ignore;
    }
    return DirEntryAction::ReadFile;
}


fn search_directory_for_main_function<P: AsRef<Path> + Clone, C: Fn(&DirEntry) -> DirEntryAction>(
    search_dir: P,
    dir_entry_checker: C,
) -> Result<PathBuf, MainFnSearchError> {
    return search_directory_for_main_function_recursor(search_dir, Rc::new(dir_entry_checker), 0);
}

fn search_directory_for_main_function_recursor<P: AsRef<Path> + Clone, C: Fn(&DirEntry) -> DirEntryAction>(
    search_dir: P,
    dir_entry_checker: Rc<C>,
    recursion_level: usize
) -> Result<PathBuf, MainFnSearchError> {
    let entries = fs::read_dir(&search_dir)?;
    for entry in entries {
        let entry = match entry {
            Err(e) => {
                error!("Failed to read entry: {}", e);
                continue;
            },
            Ok(entry) => entry,
        };
        let entry_path = entry.path();
        match dir_entry_checker(&entry) {
            DirEntryAction::Ignore => continue,
            DirEntryAction::ReadDir => {
                if recursion_level > 3 {
                    error!("Attempting too much recursion, skipping check of path: {}", entry_path.display());
                    continue;
                }
                let subdir_result = search_directory_for_main_function_recursor(entry_path.clone(), dir_entry_checker.clone(), recursion_level + 1);
                if subdir_result.is_ok() {
                    return subdir_result;
                }
            },
            DirEntryAction::ReadFile => {
                match fs::read_to_string(&entry_path) {
                    Err(err) => {
                        error!("Failed to check file {}: {}", entry_path.display(), err);
                        continue;
                    },
                    Ok(content) => {
                        // Naive check for a main function
                        if !content.contains("fn main(") {
                            continue;
                        }
                        return Ok(entry_path);
                    }
                }
            },
        }
    }

    let base_path = search_dir.as_ref().to_path_buf();
    Err(MainFnSearchError::MainNotFound(base_path))
}

