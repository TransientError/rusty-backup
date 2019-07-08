use std::path::{Path, PathBuf};
use std::io::BufReader;
use std::fs::File;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub package_managers: Vec<PackageManager>,
    pub updaters: Vec<Updater>
}

#[derive(Deserialize, Debug)]
pub struct PackageManager {
    pub name: String,
    pub list: String
}

impl PackageManager {
    pub fn get_path(&self) -> PathBuf {
        return Path::new(&format!("archives/{}.txt", &self.name)).to_owned()
    }
}

#[derive(Deserialize, Debug)]
pub struct Updater {
    pub name: String,
    pub custom: Option<String>,
    pub destination: Option<String>,
    pub credentials: Option<String>
}

pub fn read_config(config_path: &str) -> AppConfig {
    return serde_json::from_reader(
        BufReader::new(File::open(config_path).expect("error on config file open")))
    .expect("error on config deserialization");
}
