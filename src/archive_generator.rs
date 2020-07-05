use crate::appconfig::Archive;
use crate::Result;

use failure::{bail, format_err};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::result;

pub fn generate_archive(package: &Archive, archive_path: &String) -> Result<()> {
    let path = build_path(archive_path, &package.name);
    match Command::new("sh").arg("-c").arg(&package.content).output() {
        result::Result::Ok(ref output) if path.exists() => overwrite_if_different(output, path.as_path()),
        result::Result::Ok(output) if output.status.success() => {
            fs::write(path, output.stdout).map_err(failure::Error::from)
        }
        result::Result::Ok(output) => bail!("{:?}", output.stderr),
        result::Result::Err(e) => Err(failure::Error::from(e)),
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

fn build_path(archive_path: &String, filename: &String) -> PathBuf {
    Path::new(archive_path).join(filename).with_extension("txt").to_owned()
}

#[cfg(test)]
mod tests {

    use super::*;
    use tempdir::TempDir;

    #[test]
    fn error() {
        let archive = Archive {
            name: "test".to_owned(),
            content: "blah".to_owned(),
        };

        let result = generate_archive(&archive, &String::from("."));

        assert!(result.is_err());
    }

    #[test]
    fn write() {
        let archive = Archive {
            name: "test".to_owned(),
            content: "echo hi".to_owned(),
        };

        let tmp_dir = TempDir::new("write_tmp").unwrap();
        let result = generate_archive(&archive, &tmp_dir.path().to_string_lossy().into_owned());

        assert!(result.is_ok());
        assert!(tmp_dir.path().join("test.txt").exists());
    }

    #[test]
    fn overwrite() {
        let archive = Archive {
            name: "test".to_owned(),
            content: "echo something".to_owned(),
        };

        let tmp_dir = TempDir::new("test").unwrap();
        let file = tmp_dir.path().join("test.txt");
        fs::write(file, "hi").unwrap();

        let result = generate_archive(&archive, &tmp_dir.path().to_string_lossy().into_owned());

        assert!(result.is_ok());
    }
}
