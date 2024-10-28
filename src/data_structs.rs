use std::env; 
use std::process::Command;
use std::path::PathBuf;
use std::fs;

use utility::{info, error, path_move};

pub struct Kinoko {
    pub argv: Vec<String>,
    pub argc: usize,
    pub cwd: PathBuf,
}

#[derive(Debug)]
pub enum GerminationError {
    MissingRoots(PathBuf),
    MushroomUnpickable(PathBuf),
    InvalidRoot(String),
    NoHeadDir(std::io::Error),
    GrowthFailure(String),
}
impl std::fmt::Display for GerminationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
	match self {
	    GerminationError::MissingRoots(path) => {
		write!(f, "No kinoko file is found at cwd: {}", path.display())
	    },
	    GerminationError::MushroomUnpickable(path) => {
		write!(f, "Unable to pick mushroom's information: {}", path.display())
	    },
	    GerminationError::InvalidRoot(root) => {
		write!(f, "Mushroom is unhealthy to be picked, doesn't seem to be a file: {}", root)
	    },
	    GerminationError::NoHeadDir(io_error) => {
		write!(f, "Failed to create the space for germination, folder creation failed: {}", io_error)
	    },
	    GerminationError::GrowthFailure(failure) => {
		write!(f, "Germination has failed to be done - {}", failure)
	    },
	}
    }
}
impl std::error::Error for GerminationError {}

impl Kinoko {
    pub fn new(cwd: PathBuf) -> Self {
        let mut argv:Vec<String> = env::args().collect();
        let argc:usize = argv.len();
        return Kinoko {
            argv: argv,
            argc: argc,
            cwd: cwd,
        };
    }

    pub fn new_with_args(cwd: PathBuf, args: Vec<String>) -> Self {
	let argc = args.len();
	return Kinoko {
	    argv: args,
	    argc: argc,
	    cwd: cwd,
	};
    }

    pub fn get_mushroom_path(&self) -> PathBuf {
        let mushroom_path = self.cwd.join("kinoko.🍄");
        return mushroom_path;
    }

    pub fn has_roots_at_cwd(&self) -> bool {
        return self.get_mushroom_path().is_file();
    }

    pub fn get_mushroom_head_path(&self, mushroom: &Mushroom) -> PathBuf {
        let mut m_head = mushroom.head.clone();
        #[cfg(target_family="windows")]
        { m_head.push_str(".exe"); }
        let path = self.cwd.join(&m_head);
        return path;
    }

    pub fn get_mushroom_old_head_path(&self, mushroom: &Mushroom) -> PathBuf {
        let mut m_head = mushroom.head.clone();
        m_head.push_str(".old");
        #[cfg(target_family="windows")]
        { m_head.push_str(".exe"); }
        let path = self.cwd.join(&m_head);
        return path;
    }

    pub fn mushroom_head_exists(&self, mushroom: &Mushroom) -> bool {
        let head_path = self.get_mushroom_head_path(mushroom);
        return head_path.is_file();
    }

    pub fn try_germinate(&self) -> Result<PathBuf, GerminationError> {
        if ! self.has_roots_at_cwd() {
            return Err(GerminationError::MissingRoots(self.cwd.clone()));
        }
        let mushroom_path = self.get_mushroom_path();
        let mushroom = Mushroom::deserialize(&mushroom_path);
        let mushroom = match mushroom {
            Some(v) => v,
            None => return Err(GerminationError::MushroomUnpickable(mushroom_path)),
        };
        info!("Mushroom.root = {}", mushroom.root);
        info!("Mushroom.head = {}", mushroom.head);
        let source_path = self.cwd.join(&mushroom.root);
        if ! source_path.is_file() {
	   return Err(GerminationError::InvalidRoot(mushroom.root));
        }

        let target_path = self.cwd.join(&mushroom.head);
        let target_dir  = target_path.parent();
        if let Some(target_dir) = target_dir {
            match fs::create_dir_all(target_dir) {
                Ok(_) => {},
                Err(err) => {
                    return Err(GerminationError::NoHeadDir(err));
                },
            }
        }

        return match try_make_head_from_roots(&mushroom, &self) {
            Ok(_) => Ok(if cfg!(windows) { PathBuf::from(format!("{}", target_path.display()).replace("/", "\\")) } else { target_path }),
            Err(msg) => Err(GerminationError::GrowthFailure(msg)),
        };
    }

    pub fn germinate(&self) -> bool {
	return match self.try_germinate() {
	    Ok(_) => true,
	    Err(e) => {
		error!("{}", e);
		return false;
	    },
	};
    }
}
pub struct Mushroom {
    pub root: String,
    pub head: String,
}
impl Mushroom {
    pub fn new() -> Mushroom {
        Mushroom {
            root: String::new(),
            head: String::new(),
        }
    }

    pub fn serialize(&self) -> String {
        return format!("root: {}\nhead: {}", self.root, self.head);
    }

    pub fn deserialize<P: AsRef<std::path::Path>>(file: P) -> Option<Mushroom> {
        let file = file.as_ref().to_path_buf();
        if !file.is_file() {
            return None;
        }
        return match fs::read_to_string(file) {
            Err(_) => None,
            Ok(contents) => {
                let mut mushroom = Mushroom::new();
                let mut has_root = false;
                let mut has_head = false;
                for line in contents.lines() {
                    let split = match line.split_once(':') {
                        None => continue,
                        Some(key_val) => key_val,
                    };
                    let key = split.0.trim();
                    let val = split.1.trim();
                    match key {
                        "root" => {
                            mushroom.root = String::from(val);
                            has_root = true;
                        },
                        "head" => {
                            mushroom.head = String::from(val);
                            has_head = true;
                        },
                        _ => {}
                    };
                }
                if ! has_root {
                    error!("Mushroom has no root! Root is required to know where main function is located");
                    return None;
                }
                if ! has_head {
                    error!("Mushroom has no head! Defaulting to build/app");
                    #[cfg(target_family="windows")]
                    { mushroom.head = String::from("build\\app") };
                    #[cfg(target_family="unix")]
                    { mushroom.head = String::from("build/app") };
                }

                Some(mushroom)
            }
        };
    }

    pub fn create_command(&self, kinoko: &Kinoko) -> Command {
        let mut cmd = Command::new("rustc");
        cmd.arg("-o").arg({
            let mut output = kinoko.cwd.join(&self.head);
            #[cfg(target_family="windows")]
            output.set_extension("exe");
            output
        }).arg({
            kinoko.cwd.join(&self.root)
        });
        for arg in &kinoko.argv {
            cmd.arg(&arg);
        }

        cmd
    }
}

fn try_make_head_from_roots(mushroom: &Mushroom, kinoko: &Kinoko) -> Result<(), String> {
    if kinoko.mushroom_head_exists(&mushroom) {
        let mhead_path = kinoko.get_mushroom_head_path(&mushroom);
        let old_mhead_path = kinoko.get_mushroom_old_head_path(&mushroom);
        path_move(mhead_path, old_mhead_path);
    }
    let mut cmd = mushroom.create_command(kinoko);
    let result = cmd.status();
    let mut succeeded = false;
    return match result {
        Err(err) => {
            restore_old_mushroom_head_if_exists(&mushroom, &kinoko);

	   Err(format!("Failed to execute command: {}", err))
        },
        Ok(status) => {
            if status.success() {
                #[cfg(target_family="windows")]
                info!("Germinated succesfully: {}.exe", mushroom.head);
                #[cfg(target_family="unix")]
                info!("Germinated succesfully: {}", mushroom.head);

                Ok(())
            } else {
                restore_old_mushroom_head_if_exists(&mushroom, &kinoko);

                Err(format!("Command failed: No germination done"))
            }
        },
    };
}
fn make_head_from_roots(mushroom: &Mushroom, kinoko: &Kinoko) -> bool {
    match try_make_head_from_roots(mushroom, kinoko) {
	Ok(_) => true,
	Err(e) => {
	    error!("{}", e);

	    false
	},
    }
}

pub fn restore_old_mushroom_head_if_exists(mushroom: &Mushroom, kinoko: &Kinoko) -> bool {
    let old_mhead_path = kinoko.get_mushroom_old_head_path(&mushroom);
    if ! old_mhead_path.is_file() {
        return false;
    }
    let mhead_path = kinoko.get_mushroom_head_path(&mushroom);
    info!("Restoring old head...");
    return path_move(old_mhead_path, mhead_path);
}

