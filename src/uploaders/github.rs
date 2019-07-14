use std::path::Path;
use serde_json::{Value, json};
use std::fs;
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
    let token = backups.get_creds().ok_or(format_err!("Github OAuth token required for Github backup"))?;
    let gist_id = backups.get_destination().ok_or(format_err!("Gist id required for Github backup"))?;
    let client = reqwest::Client::new();

    match client.patch(&format!("https://api.github.com/gists/{}", gist_id))
        .json(&get_request_map(path))
        .bearer_auth(token)
        .send() {
            Ok(_) => Ok(()),
            Err(e) => Err(failure::Error::from(e))
        }
}
