use std::path::Path;
use serde_json::{Value, json};
use std::fs;
use std::borrow::Cow;
use reqwest::Response;

use crate::appconfig::Backup;
use crate::Result;
use failure::format_err;

fn get_request_map(path: &Path) -> Value {
    let package_name = Path::file_stem(path).and_then(|s| s.to_str()).unwrap();
    let filename = format!("{}.txt", package_name);
    let content = fs::read_to_string(path).unwrap();

    json!({
        "description": "backup for kvwu",
        "files": json!({
            filename.clone(): json!({
                "content": content,
                "filename": filename
            })
        })
    })
}

pub fn upload(path: &Path, backups: &Backup) -> Result<()> {
    let token = backups.get_creds()
        .ok_or(format_err!("Github OAuth token required for Github backup; did not backup {}", get_file_name(path)))?;

    let gist_id = backups.get_destination()
        .ok_or(format_err!("Gist id required for Github backup; did not backup {}", get_file_name(path)))?;

    let client = reqwest::Client::new();

    match client.patch(&format!("https://api.github.com/gists/{}", gist_id))
        .json(&get_request_map(path))
        .bearer_auth(token)
        .send() {
            Ok(ref r) if r.status().is_success() => Ok(()),
            Ok(r) => Err(format_err!("{}", get_text(r))),
            Err(e) => Err(failure::Error::from(e))
        }
}

fn get_file_name(path: &Path) -> Cow<str> {
    match path.file_name() {
        Some(s) => s.to_string_lossy(),
        None => Cow::from("")
    }
}

fn get_text(mut r: Response) -> String {
    match r.text() {
        Ok(s) => s,
        Err(e) => format!("{}", e)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_validate_before_calling() {
        let backup = Backup {
            name: "test".to_owned(),
            custom: None,
            credentials: None,
            destination: Some("somewhere".to_owned())
        };

        let path = Path::new("path");

        let result = upload(path, &backup);

        assert!(result.is_err());
    }

    #[test]
    fn should_upload() {
        // Will have to get some DI in this shit first
    }

}
