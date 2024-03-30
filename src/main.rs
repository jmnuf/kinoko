use std::env; 
use std::process::{ExitCode, Command};
use std::path::{Path,PathBuf};
use std::fs::{self, File, DirEntry};
use std::io::{self, Write};
use std::rc::Rc;
use std::ffi::OsStr;

macro_rules! info {
    ($($x:expr),*) => { println!("[INFO] {}", format!($($x),*)) }
}
macro_rules! error {
    ($($x:expr),*) => { eprintln!("[ERROR] {}", format!($($x),*)) }
}

static mut CWD : Option<PathBuf> = None;
fn get_cwd() -> PathBuf {
    let cwd = unsafe { CWD.clone().unwrap() };
    return cwd;
}

struct Kinoko {
    program: String,
    argv: Vec<String>,
    argc: usize,
    cwd: PathBuf,
}
impl Kinoko {
    fn new(cwd: PathBuf) -> Kinoko {
        let mut argv:Vec<String> = env::args().collect();
        let program = argv.remove(0);
        let argc:usize = argv.len();
        return Kinoko {
            program: program,
            argv: argv,
            argc: argc,
            cwd: cwd,
        };
    }

    fn print_usage(&self) {
        println!("{} ", self.program);
    }

    fn get_mushroom_path(&self) -> PathBuf {
        let mushroom_path = self.cwd.join("kinoko.🍄");
        return mushroom_path;
    }

    fn has_roots_at_cwd(&self) -> bool {
        return self.get_mushroom_path().is_file();
    }

    fn get_mushroom_head_path(&self, mushroom: &Mushroom) -> PathBuf {
        let mut m_head = mushroom.head.clone();
        #[cfg(target_family="windows")]
        { m_head.push_str(".exe"); }
        let path = self.cwd.join(&m_head);
        return path;
    }

    fn get_mushroom_old_head_path(&self, mushroom: &Mushroom) -> PathBuf {
        let mut m_head = mushroom.head.clone();
        m_head.push_str(".old");
        #[cfg(target_family="windows")]
        { m_head.push_str(".exe"); }
        let path = self.cwd.join(&m_head);
        return path;
    }

    fn mushroom_head_exists(&self, mushroom: &Mushroom) -> bool {
        let head_path = self.get_mushroom_head_path(mushroom);
        return head_path.is_file();
    }

    fn try_to_germinate(&self) -> bool {
        if ! self.has_roots_at_cwd() {
            return false;
        }
        let mushroom = Mushroom::deserialize(self.get_mushroom_path());
        let mushroom = match mushroom {
            Some(v) => v,
            None => return false,
        };
        info!("Mushroom.root = {}", mushroom.root);
        info!("Mushroom.head = {}", mushroom.head);
        let source_path = self.cwd.join(&mushroom.root);
        if ! source_path.is_file() {
            error!("Source file doesn't exist: {}", mushroom.root);
            return false;
        }
        let target_path = self.cwd.join(&mushroom.head);
        let target_dir  = target_path.parent();
        if let Some(target_dir) = target_dir {
            match fs::create_dir_all(target_dir) {
                Ok(_) => {},
                Err(err) => {
                    error!("{}", err);
                    return false;
                },
            }
        }
        return make_head_from_roots(&mushroom, &self);
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

    fn create_command(&self, kinoko: &Kinoko) -> Command {
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

fn path_move<P1: AsRef<Path>, P2: AsRef<Path>>(original_path: P1, new_path: P2) -> bool {
    let original_path = original_path.as_ref();
    let new_path = new_path.as_ref();
    let shared_prefix = get_cwd().shared_prefix_with(vec![&original_path, &new_path]);
    let result = fs::rename(&original_path, &new_path);
    let (original_path, new_path) = if let Some(prefix) = shared_prefix {
        let p1 = original_path.strip_prefix(&prefix).unwrap();
        let p2 = new_path.strip_prefix(&prefix).unwrap();
        (p1, p2)
    } else { (original_path, new_path) };
    return match &result {
        Err(err) => {
            error!("Failed to move {} -> {}: {}", original_path.display(), new_path.display(), err);
            false
        },
        Ok(_) => {
            info!("{} -> {}", original_path.display(), new_path.display());
            true
        },
    }
}

fn restore_old_mushroom_head_if_exists(mushroom: &Mushroom, kinoko: &Kinoko) -> bool {
    let old_mhead_path = kinoko.get_mushroom_old_head_path(&mushroom);
    if ! old_mhead_path.is_file() {
        return false;
    }
    let mhead_path = kinoko.get_mushroom_head_path(&mushroom);
    info!("Restoring old head...");
    return path_move(old_mhead_path, mhead_path);
}

fn make_head_from_roots(mushroom: &Mushroom, kinoko: &Kinoko) -> bool {
    if kinoko.mushroom_head_exists(&mushroom) {
        let mhead_path = kinoko.get_mushroom_head_path(&mushroom);
        let old_mhead_path = kinoko.get_mushroom_old_head_path(&mushroom);
        path_move(mhead_path, old_mhead_path);
    }
    let mut cmd = mushroom.create_command(kinoko);
    let result = cmd.status();
    let mut succeeded = false;
    match result {
        Err(err) => {
            error!("Failed to execute command: {}", err);
            restore_old_mushroom_head_if_exists(&mushroom, &kinoko);
        },
        Ok(status) => {
            if status.success() {
                #[cfg(target_family="windows")]
                info!("Germinated succesfully: {}.exe", mushroom.head);
                #[cfg(target_family="unix")]
                info!("Germinated succesfully: {}", mushroom.head);
                succeeded = true;
            } else {
                error!("Failed to germinate");
                restore_old_mushroom_head_if_exists(&mushroom, &kinoko);
            }
        },
    };
    return succeeded;
}

fn main() -> ExitCode {
    let cwd = match env::current_dir() {
        Ok(value) => value,
        Err(err) => {
            error!("Spores failed to reach current directory: {}", err);
            return ExitCode::FAILURE;
        },
    };
    unsafe {
        CWD = Some(cwd.clone());
    }
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

trait KinokoPath {
    fn with_rename<S: AsRef<OsStr>>(&self, new_name: S) -> PathBuf;
    fn shared_prefix_with<P: AsRef<Path>>(&self, others: Vec<P>) -> Option<PathBuf>;
}
impl<C: AsRef<Path>> KinokoPath for C {
    fn with_rename<S: AsRef<OsStr>>(&self, new_name: S) -> PathBuf {
        let path = self.as_ref();
        let mut result = path.to_owned();
        result.set_file_name(new_name);
        if let Some(ext) = path.extension() {
            result.set_extension(ext);
        }
        return result;
    }

    fn shared_prefix_with<P: AsRef<Path>>(&self, others: Vec<P>) -> Option<PathBuf> {
        let base_path = path_comps_to_vec(self.as_ref().components());
        let mut prefix = PathBuf::new();
        let mut found = false;
        for (idx, left) in base_path.iter().enumerate() {
            let mut matched = true;
            for other_path in others.iter() {
                let right_components = path_comps_to_vec(other_path.as_ref().components());
                let right = match right_components.get(idx) {
                    None => {
                        matched = false;
                        break;
                    },
                    Some(r) => r,
                };
                if left != right {
                    matched = false;
                    break;
                }
            }
            if ! matched {
                break;
            }
            prefix.push(left.as_os_str());
            found = true;
        }

        return if found { Some(prefix) } else { None };
    }
}

fn path_comps_to_vec(components: std::path::Components) -> Vec<std::path::Component> {
    let mut v = Vec::new();
    for c in components {
        v.push(c);
    }
    return v;
}
