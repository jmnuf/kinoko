use std::env; 
use std::process::Command;
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{Write};

macro_rules! info {
    ($($x:expr),*) => { println!("[INFO] {}", format!($($x),*)); }
}
macro_rules! error {
    ($($x:expr),*) => { eprintln!("[ERROR] {}", format!($($x),*)); }
}

struct Kinoko {
    program: String,
    argv: Vec<String>,
    argc: usize,
    os_temp_dir: PathBuf,
    cwd: PathBuf,
}
impl Kinoko {
    fn new(cwd: PathBuf) -> Kinoko {
        let mut argv:Vec<String> = env::args().collect();
        let program = argv.remove(0);
        let argc:usize = argv.len();
        let temp_dir = env::temp_dir();
        return Kinoko {
            program: program,
            argv: argv,
            argc: argc,
            os_temp_dir: temp_dir,
            cwd: cwd,
        };
    }

    fn pop_arg(&mut self) -> Option<String> {
        if self.argc < 1 {
            return None;
        }
        let item = self.argv.remove(0);
        self.argc -= 1;
        return Some(item);
    }

    fn get_mushroom_path(&self) -> PathBuf {
        let mushroom_path = self.cwd.join("kinoko.🍄");
        return mushroom_path;
    }

    fn has_roots_at_cwd(&self) -> bool {
        return self.get_mushroom_path().is_file();
    }
}
struct Mushroom {
    root: String,
    head: String,
}
impl Mushroom {
    fn new() -> Mushroom {
        Mushroom {
            root: String::new(),
            head: String::new(),
        }
    }

    fn serialize(&self) -> String {
        return format!("root: {}\nhead: {}", self.root, self.head);
    }

    fn deserialize(file: PathBuf) -> Option<Mushroom> {
        if !file.is_file() {
            return None;
        }
        return match fs::read_to_string(file) {
            Err(_) => None,
            Ok(contents) => {
                let mut mushroom = Mushroom::new();
                for line in contents.lines() {
                    let split = match line.split_once(':') {
                        None => continue,
                        Some(key_val) => key_val,
                    };
                    let key = split.0.trim();
                    let val = split.1.trim();
                    match key {
                        "root" => mushroom.root = String::from(val),
                        "head" => mushroom.head = String::from(val),
                        _ => {}
                    };
                }

                Some(mushroom)
            }
        };
    }

    fn create_command(&self, kinoko: &Kinoko) -> Command {
        let mut cmd = Command::new("rustc");
        cmd.arg("-o").arg({
            kinoko.cwd.join(&self.head)
        }).arg({
            kinoko.cwd.join(&self.root)
        });
        for arg in &kinoko.argv {
            cmd.arg(&arg);
        }

        cmd
    }
}


fn main() {
    let cwd = match env::current_dir() {
        Ok(value) => value,
        Err(err) => {
            error!("Spores failed to reach current directory: {}", err);
            return;
        },
    };
    let kinoko = Kinoko::new(cwd);
    if kinoko.has_roots_at_cwd() {
        let mushroom = Mushroom::deserialize(kinoko.get_mushroom_path());
        if let Some(mushroom) = mushroom {
            info!("Mushroom.root = {}", mushroom.root);
            info!("Mushroom.head = {}", mushroom.head);
            let mut cmd = mushroom.create_command(&kinoko);
            let result = cmd.status();
            match result {
                Err(err) => error!("Failed to execute command: {}", err),
                Ok(status) => {
                    if status.success() {
                        info!("Germinated succesfully");
                    }
                },
            }
            return;
        }
    }
    info!("Spores thrown at: {}", kinoko.cwd.display());
    let entries = match kinoko.cwd.read_dir() {
        Ok(dir) => dir,
        Err(err) => {
            error!("Failed to spread roots in present directory: {}", err);
            return;
        }
    };
    for entry in entries {
        if let Err(e) = entry {
            error!("{}", e);
            continue;
        }
        let entry_path = entry.unwrap().path();
        if !entry_path.is_file() {
            continue;
        }
        if entry_path != entry_path.with_extension("rs") {
            continue;
        }
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
            }
        }
        match File::create(kinoko.get_mushroom_path()) {
            Err(err) => {
                error!("Failed to grow mushroom: {}", err);
            },
            Ok(mut file) => {
                let target_name = match entry_path.parent() {
                    Some(parent_dir) => parent_dir.join(parent_dir.file_stem().unwrap()).with_extension("exe"),
                    None => entry_path.with_extension("exe"),
                };
                let prefix = format!("{}\\", kinoko.cwd.to_str().unwrap());
                let entry_path = format!("{}", entry_path.display());
                let entry_path = entry_path.strip_prefix(&prefix).unwrap();
                let target_name = format!("{}", target_name.display());
                let target_name = target_name.strip_prefix(&prefix).unwrap();
                let mut mushroom = Mushroom::new();
                mushroom.root = entry_path.to_string();
                mushroom.head = target_name.to_string();
                let contents:String = mushroom.serialize();
                match file.write(&contents.into_bytes()) {
                    Ok(_) => info!("Mushroom sprung"),
                    Err(err) => error!("Failed to spring mushroom: {}", err),
                }
            }
        };
        break;
    }
}
