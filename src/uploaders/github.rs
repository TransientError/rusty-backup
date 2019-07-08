use std::path::Path;
use serde_json::{Value, json};
use std::fs;
use crate::appconfig::Updater;
use super::StorageService;

pub struct GithubStorageService {}

impl GithubStorageService {
    fn get_request_map(&self, path: &Path) -> Value {
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
}

impl StorageService for GithubStorageService {
    fn upload(&self, path: &Path, updater: &Updater) {
        let token = updater.credentials.as_ref().expect("Github OAuth token required for Github backup");
        let gist_id = updater.destination.as_ref().expect("Gist id required for Github backup");
        let client = reqwest::Client::new();

        client.patch(&format!("https://api.github.com/gists/{}", gist_id))
            .json(&self.get_request_map(path))
            .bearer_auth(token)
            .send().unwrap();
    }
}
