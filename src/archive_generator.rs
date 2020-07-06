use crate::appconfig::Archive;
use anyhow::{bail, Error, Result};
use tokio::process::Command;

use std::{
    path::{Path, PathBuf},
    process::Output,
};
use tokio::fs;

pub async fn generate_archive(package: &Archive, archive_path: &String) -> Result<()> {
    let path = build_path(archive_path, &package.name);

    match Command::new("sh").arg("-c").arg(&package.content).output().await {
        Ok(ref output) if path.exists() => overwrite_if_different(output, path.as_path()).await,
        Ok(output) if output.status.success() => fs::write(path, output.stdout).await.map_err(Error::from),
        Ok(output) => bail!("{:?}", output.stderr),
        Err(e) => Err(Error::from(e)),
    }
}

async fn overwrite_if_different(output: &Output, path: &Path) -> Result<()> {
    let error_message = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() || !error_message.is_empty() {
        bail!("Error while trying to archive: {}", error_message)
    }
    let old_content = fs::read_to_string(path).await.unwrap_or_else(|_e| "".to_string());
    let new_content = String::from_utf8_lossy(&output.stdout);
    if old_content != new_content {
        fs::write(path, new_content.as_ref()).await?
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

    #[tokio::test]
    async fn error() {
        let archive = Archive {
            name: "test".to_owned(),
            content: "blah".to_owned(),
        };

        let result = generate_archive(&archive, &String::from(".")).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn write() {
        let archive = Archive {
            name: "test".to_owned(),
            content: "echo hi".to_owned(),
        };

        let tmp_dir = TempDir::new("write_tmp").unwrap();
        let result = generate_archive(&archive, &tmp_dir.path().to_string_lossy().into_owned()).await;

        assert!(result.is_ok());
        assert!(tmp_dir.path().join("test.txt").exists());
    }

    #[tokio::test]
    async fn overwrite() {
        let archive = Archive {
            name: "test".to_owned(),
            content: "echo something".to_owned(),
        };

        let tmp_dir = TempDir::new("test").unwrap();
        let file = tmp_dir.path().join("test.txt");
        fs::write(file, "hi").await.unwrap();

        let result = generate_archive(&archive, &tmp_dir.path().to_string_lossy().into_owned()).await;

        assert!(result.is_ok());
    }
}
