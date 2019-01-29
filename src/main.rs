#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate reqwest;

use std::fs::{self, File, DirEntry};
use std::io::{BufReader};
use std::process::Command;
use std::path::{Path, PathBuf};
use std::time::{Duration};
use serde_json::{Value, json};

const CONFIG_PATH: &str = "./backup.json";

fn main() {
    let config = read_config(CONFIG_PATH);
    println!("{:?}", config)
}

fn read_config(config_path: &str) -> AppConfig {
    return serde_json::from_reader(
        BufReader::new(File::open(config_path).expect("error on config file open")))
    .expect("error on config deserialization");
}

#[derive(Deserialize, Debug)]
struct AppConfig {
    package_managers: Vec<PackageManager>,
    updaters: Vec<Updater>
}

fn generate_archive(package: &PackageManager) {
    let path = package.get_path();
    match (Command::new(&package.list).output(), path.exists()) {
        (Result::Ok(output), true) => overwrite_if_different(output.stdout, path.as_path()),
        (Result::Ok(output), false) => fs::write(path, output.stdout).unwrap_or(()),
        _ => return 
    }
}

fn overwrite_if_different(new: Vec<u8>, path: &Path) {
    let old_content = fs::read_to_string(path).unwrap_or("".to_owned());
    let new_content = String::from_utf8(new).unwrap_or("".to_owned());
    if old_content != new_content {
        fs::write(path, new_content).unwrap_or(())
    }
}

#[derive(Deserialize, Debug)]
struct PackageManager {
    name: String,
    list: String
}

impl PackageManager {
    fn get_path(&self) -> PathBuf {
        return Path::new(&format!("archives/{}-packages.txt", &self.name)).to_owned()
    }
}

fn mapStorageServices(name: &String) -> Box<StorageService> {
    match name.as_ref() {
        "github-gists" => Box::new(GithubStorageService { }),
        _ => Box::new(CustomStorageService {})
    }
}

fn perform_backup(updaters: Vec<Updater>) {
    let entries = fs::read_dir(Path::new("archives")).expect("Failed to read archives");
    entries.filter(Result::is_ok)
        .map(Result::unwrap)
        .filter(has_been_modified)
        .map(|e| e.path())
        .for_each(|p| updaters.iter().for_each(|u| mapStorageServices(&u.name).as_ref().upload(p.as_ref(), &u)));
}

fn has_been_modified(entry: &DirEntry) -> bool {
    let one_day_secs = 86400;
    let modified = entry.metadata()
        .and_then(|m| m.modified());

    match modified {
        Result::Ok(m) => m.elapsed().map(|dur| dur < Duration::from_secs(one_day_secs)).unwrap_or(false),
        _ => false
    }
}

trait StorageService {
    fn upload(&self, path: &Path, updater: &Updater);
}

#[derive(Deserialize, Debug)]
struct Updater {
    name: String,
    custom: Option<String>,
    destination: Option<String>,
    credentials: Option<String>
}

struct CustomStorageService { }

impl StorageService for CustomStorageService {
    fn upload(&self, path: &Path, updater: &Updater) {
        Command::new(format!("cat {} | {}", 
            path.to_str().unwrap_or_default(), 
            updater.custom.as_ref().expect("Custom Storage Service requires a custom command to run")));
    }
}

struct GithubStorageService {}

impl GithubStorageService {
    fn getRequestMap(&self, path: &Path) -> Value {
        let package_name = Path::file_stem(path).and_then(|s| s.to_str()).unwrap();
        let filename = format!("{}.txt", package_name);
        let content = fs::read_to_string(path).unwrap();

        json!({
            "description": format!("backup for {}", package_name),
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

        // client.patch(format!("https://api.github.com/gists/{}", gist_id))
        //     .json(self.getRequestMap())
        //     .send()
        client.patch(&format!("https://api.github.com/gists/{}", gist_id))
            .json(&self.getRequestMap(path))
            .bearer_auth(token)
            .send().unwrap();
    }
}
