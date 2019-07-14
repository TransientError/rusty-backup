use std::process::Command;
use std::path::Path;
use std::borrow::Cow;

use crate::appconfig::Backup;
use crate::Result;
use failure::ResultExt;

pub fn upload(path: &Path, backups: &Backup) -> Result<()> {
    match Command::new("sh").arg("-c").arg(&backups.custom.as_ref().unwrap()).output() {
        Ok(ref output) if !output.status.success() || !output.stderr.is_empty() => Ok(()),
        Ok(_) => Ok(()),
        Err(e) => Err(failure::Error::from(e))
            .context(format!("{} backup failed for {}", &backups.name, get_file_name(path)))?
    }
}

fn get_file_name(path: &Path) -> Cow<str> {
    match path.file_name() {
        Some(p) => p.to_string_lossy(),
        None => Cow::from("")
    }
}
