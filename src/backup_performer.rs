use crate::uploaders::{CustomStorageService, StorageService};
use crate::uploaders::github::GithubStorageService;
use crate::appconfig::Updater;

use std::path::Path;
use std::fs::{self, DirEntry};
use std::time::Duration;

fn map_storage_services(name: &String) -> Box<StorageService> {
    match name.as_ref() {
        "github-gists" => Box::new(GithubStorageService { }),
        _ => Box::new(CustomStorageService {})
    }
}

pub fn perform_backup(updaters: Vec<Updater>) {
    let entries = fs::read_dir(Path::new("archives")).expect("Failed to read archives");
    entries.filter(Result::is_ok)
        .map(Result::unwrap)
        .filter(has_been_modified)
        .map(|e| e.path())
        .for_each(|p| updaters.iter().for_each(|u| map_storage_services(&u.name).as_ref().upload(p.as_ref(), &u)));
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