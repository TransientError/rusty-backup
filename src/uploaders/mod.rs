use std::process::Command;
use std::path::Path;
use crate::appconfig::Updater;

pub mod github;

pub trait StorageService {
    fn upload(&self, path: &Path, updater: &Updater);
}

pub struct CustomStorageService { }

impl StorageService for CustomStorageService {
    fn upload(&self, _path: &Path, updater: &Updater) {
        Command::new("sh").arg("-c").arg(&updater.custom.as_ref().unwrap())
            .output()
            .expect("Failed to run custom command");
    }
}
