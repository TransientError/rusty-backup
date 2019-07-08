use crate::appconfig::PackageManager;

use std::process::Command;
use std::fs;
use std::path::Path;

pub fn generate_archive(package: &PackageManager) {
    let path = package.get_path();

    match (Command::new("sh").arg("-c").arg(&package.list).output(), path.exists()) {
        (Result::Ok(output), true) => overwrite_if_different(output.stdout, path.as_path()),
        (Result::Ok(output), false) => fs::write(path, output.stdout).unwrap_or(()),
        (Result::Err(e), _) => println!("{:?}", e)
    }
}

fn overwrite_if_different(new: Vec<u8>, path: &Path) {
    let old_content = fs::read_to_string(path).unwrap_or("".to_owned());
    let new_content = String::from_utf8(new).unwrap_or("".to_owned());
    if old_content != new_content {
        fs::write(path, new_content).unwrap_or(())
    }
}
