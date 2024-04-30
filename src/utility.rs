use std::path::{Path,PathBuf};
use std::fs;
use std::ffi::OsStr;

#[macro_export]
macro_rules! info {
    ($($x:expr),*) => { println!("[INFO] {}", format!($($x),*)) }
}
pub use info;
#[macro_export]
macro_rules! error {
    ($($x:expr),*) => { eprintln!("[ERROR] {}", format!($($x),*)) }
}
pub use error;

static mut CWD : Option<PathBuf> = None;
pub fn get_cwd() -> PathBuf {
    let cwd = unsafe { CWD.clone().unwrap() };
    return cwd;
}
pub fn set_cwd(cwd: &PathBuf) {
    unsafe {
        CWD = Some(cwd.clone());
    };
}

pub fn path_move<P1: AsRef<Path>, P2: AsRef<Path>>(original_path: P1, new_path: P2) -> bool {
    let original_path = original_path.as_ref();
    let new_path = new_path.as_ref();
    let shared_prefix = get_cwd().shared_prefix_with(vec![&original_path, &new_path]);
    let result = fs::rename(&original_path, &new_path);
    let (original_path, new_path) = if let Some(prefix) = shared_prefix {
        let p1 = original_path.strip_prefix(&prefix).unwrap();
        let p2 = new_path.strip_prefix(&prefix).unwrap();
        // block_return
        (p1, p2)
    } else {
        // block_return
        (original_path, new_path)
    };
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

pub trait KinokoPath {
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
