use std::path::PathBuf;
use std::convert::From;
use std::fs;
use std::io::Write;

use utility::*;
use data_structs::*;

type CmdResult = Result<(), String>;
const COMMAND_NAME: &'static str = "init";

pub fn check_args(argv: &Vec<String>) -> bool {
    if argv.len() < 1 {
        return false;
    }

    let cmd_name = COMMAND_NAME.to_string();
    if argv[0] == cmd_name {
        return true;
    }

    return false;
}

pub fn usage_message() -> (String, &'static str) {
    (
	format!("{} <project-name>", COMMAND_NAME),
	"Plant a mushroom! Initialize tiny rust project"
    )
}

pub fn run_command(cwd: PathBuf, mut args: Vec<String>) -> CmdResult {
    // Remove command name
    args.remove(0);

    let project_name: String;
    let project_path: PathBuf;
    if args.len() < 1 {
        let maybe_file_name = cwd.file_name();
        if maybe_file_name.is_none() {
            return Err("Failed to get current directory name".to_string());
        }
        project_name = maybe_file_name.unwrap().to_str().unwrap().to_string();
        project_path = cwd.clone();
    } else {
        project_name = args.remove(0);
        project_path = cwd.join(project_name.clone());
        if project_path.is_dir() {
            if !is_dir_empty(&project_path) {
                return Err("Directory is not empty, can't initialize project on a non-empty directory".to_string());
            }
        } else {
            create_dir(&project_path)?;
        }
    }

    let main_file_name = "main.rs";
    let source_folder_name = "src";
    let build_folder_name = "build";

    let src_path   = project_path.join(source_folder_name);
    create_dir(&src_path)?;
    let build_path = project_path.join(build_folder_name);
    create_dir(&build_path)?;
    let main_path  = src_path.join(main_file_name);
    create_file(&main_path, r#"use std::process::ExitCode;

fn run(args: Vec<String>) -> Result<(), String> {
    println!("Hello world!");
    println!("Argument count: {}", args.len());
    println!("Program Arguments: {:?}", args);
    Ok(())
}

fn main() -> ExitCode {
    let args:Vec<String> = std::env::args().collect();
    match run(args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            ExitCode::FAILURE
        }
    }
}
"#.to_string())?;

    let mut mushroom = Mushroom::new();
    mushroom.root = format!("{}/{}", source_folder_name, main_file_name);
    mushroom.head = format!("{}/{}", build_folder_name, project_name);
    let mushroom_content = mushroom.serialize();
    let kinoko = Kinoko::new(project_path.clone());
    create_file(&kinoko.get_mushroom_path(), mushroom_content)?;

    Ok(())
}

fn is_dir_empty(path: &PathBuf) -> bool {
    return path.read_dir().map(|mut x| x.next().is_none()).unwrap_or(false);
}

fn create_dir(path: &PathBuf) -> CmdResult {
    match fs::create_dir(path) {
        Err(error) => Err(format!("{}", error)),
        Ok(_) => {
            info!("Created directory: {}", path.display());
            Ok(())
        },
    }
}

fn create_file(path: &PathBuf, contents: String) -> CmdResult {
    match fs::File::create(path) {
        Err(error) => Err(format!("{}", error)),
        Ok(mut file) => {
            match file.write_all(&contents.into_bytes()) {
                Err(error) => Err(format!("{}", error)),
                Ok(_) => {
                    info!("Created file: {}", path.display());

                    Ok(())
                },
            }
        },
    }
}

