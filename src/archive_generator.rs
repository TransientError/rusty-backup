use crate::appconfig::Archive;
use crate::Result;

use std::process::{Output, Command};
use std::fs;
use std::path::Path;
use std::result;
use failure::format_err;

pub fn generate_archive(package: &Archive) -> Result<()> {
    let path = package.get_path();

    match Command::new("sh").arg("-c").arg(&package.content).output() {
        result::Result::Ok(ref output) if path.exists() => overwrite_if_different(output, path.as_path()),
        result::Result::Ok(output) => fs::write(path, output.stdout).map_err(failure::Error::from),
        result::Result::Err(e) => Err(failure::Error::from(e))
    }
}

fn overwrite_if_different(output: &Output, path: &Path) -> Result<()> {
    let error_message = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() || !error_message.is_empty() {
        return Err(format_err!("Error while trying to archive: {}", error_message));
    }
    let empty_string = "".to_owned();
    let old_content = fs::read_to_string(path).unwrap_or(empty_string);
    let new_content = String::from_utf8_lossy(&output.stdout);
    if old_content != new_content {
        fs::write(path, new_content.as_ref())?
    }
    Ok(())
}
