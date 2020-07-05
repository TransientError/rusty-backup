use std::borrow::Cow;
use std::path::Path;
use std::process::Command;

use crate::appconfig::Backup;
use anyhow::{format_err, Context, Error, Result};

pub fn upload(path: &Path, backups: &Backup) -> Result<()> {
    let cmd = &backups.get_custom().ok_or(format_err!(
        "Command is necessary to backup; skipping {}",
        get_file_name(path)
    ))?;

    match Command::new("sh")
        .arg("-c")
        .arg(format!("cat {} | {}", path.to_string_lossy(), cmd))
        .output()
    {
        Ok(ref output) if !output.status.success() || !output.stderr.is_empty() => Ok(()),
        Ok(_) => Ok(()),
        Err(e) => {
            Err(Error::from(e)).context(format!("{} backup failed for {}", &backups.name, get_file_name(path)))?
        }
    }
}

fn get_file_name(path: &Path) -> Cow<str> {
    match path.file_name() {
        Some(p) => p.to_string_lossy(),
        None => Cow::from(""),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_upload() {
        let backup = Backup {
            name: "test".to_owned(),
            custom: Some("echo".to_owned()),
            ..Default::default()
        };

        let path = Path::new("path");

        let result = upload(path, &backup);

        assert!(result.is_ok())
    }

    #[test]
    fn should_err() {
        let backup = Backup {
            name: "test".to_owned(),
            ..Default::default()
        };

        let path = Path::new("path");

        let result = upload(path, &backup);

        assert!(result.is_err());
    }
}
