use std::io::BufReader;
use std::fs::File;
use failure::ResultExt;

use crate::Result;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub archive_path: String,
    pub archives: Vec<Archive>,
    pub backups: Vec<Backup>
}

#[derive(Deserialize, Debug, Default)]
pub struct Archive {
    pub name: String,
    pub content: String
}

#[derive(Deserialize, Debug, Default)]
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
        self.credentials.clone()
    }

    pub fn get_custom(&self) -> Option<String> {
        self.custom.clone()
    }
}

pub fn read_config(config_path: &str) -> Result<AppConfig> {
    let file = File::open(config_path).context("missing config")?;
    let buf_reader = BufReader::new(file);
    serde_json::from_reader(buf_reader)
        .map_err(|err| failure::Error::from(err))
}
