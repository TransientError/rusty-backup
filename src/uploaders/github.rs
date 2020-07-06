use reqwest::Response;
use serde_json::{json, Value};
use std::borrow::Cow;
use std::fs;
use std::path::Path;

use crate::appconfig::Backup;
use anyhow::{format_err, Error, Result, bail};

fn get_request_map(path: &Path) -> Result<Value> {
    let package_name = Path::file_stem(path).and_then(|s| s.to_str()).unwrap();
    let filename = format!("{}.txt", package_name);
    let content = fs::read_to_string(path)?;

    Ok(json!({
        "description": "backup for kvwu",
        "files": json!({
            filename.clone(): json!({
                "content": content,
                "filename": filename
            })
        })
    }))
}

fn get_host_url() -> String {
    #[cfg(not(test))]
    let url = "https://api.github.com".to_owned();

    #[cfg(test)]
    let url = mockito::server_url();

    return url;
}

pub async fn upload(path: &Path, backups: &Backup) -> Result<()> {
    let token = backups.get_creds().ok_or(format_err!(
        "Github OAuth token required for Github backup; did not backup {}",
        get_file_name(path)
    ))?;

    let gist_id = backups.get_destination().ok_or(format_err!(
        "Gist id required for Github backup; did not backup {}",
        get_file_name(path)
    ))?;

    let client = reqwest::Client::new();

    match client
        .patch(&format!("{}/gists/{}", get_host_url(), gist_id))
        .json(&get_request_map(path)?)
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(ref r) if r.status().is_success() => Ok(()),
        Ok(r) => bail!("{}", get_text(r).await),
        Err(e) => Err(Error::from(e)),
    }
}

fn get_file_name(path: &Path) -> Cow<str> {
    match path.file_name() {
        Some(s) => s.to_string_lossy(),
        None => Cow::from(""),
    }
}

async fn get_text(r: Response) -> String {
    match r.text().await {
        Ok(s) => s,
        Err(e) => format!("{}", e),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use mockito::mock;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempdir::TempDir;

    #[tokio::test]
    async fn should_validate_before_calling() {
        let backup = Backup {
            name: "test".to_owned(),
            custom: None,
            credentials: None,
            destination: Some("somewhere".to_owned()),
        };

        let path = Path::new("path");

        let result = upload(path, &backup).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_upload() {
        let backup = Backup {
            name: "test".to_owned(),
            credentials: Some("creds".to_owned()),
            destination: Some("upload".to_owned()),
            ..Default::default()
        };

        let (path, _tmp) = write_test_content("upload", "content").unwrap();

        let _m = mock("PATCH", "/gists/upload")
            .with_status(200)
            .with_body("test")
            .create();

        let result = upload(&path.as_path(), &backup).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_report_err() {
        let backup = Backup {
            name: "test".to_owned(),
            credentials: Some("creds".to_owned()),
            destination: Some("report_err".to_owned()),
            ..Default::default()
        };

        let (path, _tmp) = write_test_content("report_err", "content").unwrap();

        let _m = mock("PATCH", "/gists/report_err")
            .with_status(400)
            .with_body("error")
            .create();

        let result = upload(&path.as_path(), &backup).await;

        assert!(result.is_err());
    }

    fn write_test_content(name: &str, content: &str) -> Result<(PathBuf, TempDir)> {
        let tmp_dir = TempDir::new(&format!("backup_tmp_{}", name))?;
        let path = tmp_dir.path().join("test.txt");
        let mut tmp_file = File::create(&path)?;
        writeln!(tmp_file, "{}", content)?;
        return Ok((path, tmp_dir));
    }
}
