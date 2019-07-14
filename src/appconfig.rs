use std::path::{Path, PathBuf};
use std::io::BufReader;
use std::fs::File;
use failure::ResultExt;

use crate::Result;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub archives: Vec<Archive>,
    pub backups: Vec<Backup>
}

#[derive(Deserialize, Debug)]
pub struct Archive {
    pub name: String,
    pub content: String
}

impl Archive {
    pub fn get_path(&self) -> PathBuf {
        return Path::new(&format!("/Users/kvwu/utils/backup/archives/{}.txt", &self.name)).to_owned()
    }
}

#[derive(Deserialize, Debug)]
pub struct Backup {
    pub name: String,
    pub custom: Option<String>,
    pub destination: Option<String>,
    pub credentials: Option<String>
}

impl Backup {
    pub fn get_destination(&self) -> Option<String> {
        self.destination.clone()
    }

    pub fn get_creds(&self) -> Option<String> {
        self.destination.clone()
    }
}

pub fn read_config(config_path: &str) -> Result<AppConfig> {
    let file = File::open(config_path).context("missing config")?;
    let buf_reader = BufReader::new(file);
    serde_json::from_reader(buf_reader)
        .map_err(|err| failure::Error::from(err))
}
