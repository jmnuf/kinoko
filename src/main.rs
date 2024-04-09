use std::env; 
use std::process::ExitCode;
use std::path::{Path,PathBuf};
use std::fs::{self, File, DirEntry};
use std::io::{self, Write};
use std::rc::Rc;

// Helper
mod utility;
mod data_structs;
mod commands_helper;

// Commands
mod cmd_init;

use utility::*;
use data_structs::*;

fn main() -> ExitCode {
    let cwd = match env::current_dir() {
        Ok(value) => value,
        Err(err) => {
            error!("Spores failed to reach current directory: {}", err);
            return ExitCode::FAILURE;
        },
    };
    set_cwd(&cwd);
    let kinoko = Kinoko::new(cwd.clone());
    if kinoko.argc > 0 && kinoko.argv[0] == String::from("-h") {
        kinoko.print_usage();
        return ExitCode::SUCCESS;
    }
    if kinoko.try_to_germinate() {
        return ExitCode::SUCCESS;
    }
    info!("Spores thrown at: {}", kinoko.cwd.display());
    let root_path = match search_directory_for_main_function(&kinoko.cwd, rust_file_dir_entry_checker) {
        Ok(path) => path,
        Err(err) => {
            error!("{}", err);
            return ExitCode::FAILURE;
        },
    };
    
    match File::create(kinoko.get_mushroom_path()) {
        Err(err) => {
            error!("Failed to grow mushroom: {}", err);
            return ExitCode::FAILURE;
        },
        Ok(mut file) => {
            let target_name = cwd.join("build").join(cwd.file_stem().unwrap());
            let root_path = root_path.strip_prefix(&cwd).unwrap();
            let root_path = format!("{}", root_path.display());
            let target_name = target_name.strip_prefix(&cwd).unwrap();
            let target_name = format!("{}", target_name.display());
            let mut mushroom = Mushroom::new();
            mushroom.root = root_path.to_string();
            mushroom.head = target_name.to_string();
            let contents:String = mushroom.serialize();
            match file.write(&contents.into_bytes()) {
                Ok(_) => info!("Mushroom sprung"),
                Err(err) => {
                    error!("Failed to spring mushroom: {}", err);
                    return ExitCode::FAILURE;
                },
            }
        }
    };

    if ! kinoko.try_to_germinate() {
        return ExitCode::FAILURE;
    }
    return ExitCode::SUCCESS;
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

