use crate::uploaders::{custom, github};
use crate::appconfig::Backup;
use crate::Result;
use crate::log_err;

use std::path::{PathBuf, Path};
use std::fs::{self, DirEntry};
use std::time::{SystemTime, Duration};
use std::result;
use std::io;
use log::info;
use rayon::prelude::*;

pub fn perform_backup(backups: Vec<Backup>) {
    match fs::read_dir(Path::new("/Users/kvwu/utils/backup/archives")) {
        Ok(entries) => {
            entries.filter_map(process_file)
            .filter_map(process_entry)
            .for_each(|p| process_path(p, &backups));
        },
        Err(e) => log_err(failure::Error::from(e), log::Level::Error)
    }
}

fn process_file(result: io::Result<DirEntry>) -> Option<DirEntry> {
    match result {
        Ok(dir) => Some(dir),
        Err(e) => {log_err(failure::Error::from(e), log::Level::Warn); None}
    }
}

fn process_entry(entry: DirEntry) -> Option<PathBuf> {
    let one_day_secs = 86400;
    let modified_time = entry.metadata()
        .and_then(|m| m.modified());

    match modified_time {
        result::Result::Ok(m) if has_been_modified(m, one_day_secs) => Some(entry.path()),
        result::Result::Ok(_) => {
            info!("{} has not been modified; skipping", entry.file_name().to_string_lossy());
            None
        },
        Err(e) => {
            log_err(failure::Error::from(e), log::Level::Warn);
            None
        }
    }
}

fn has_been_modified(time: SystemTime, secs: u64) -> bool {
    return time.elapsed()
        .map(|dur| dur < Duration::from_secs(secs))
        .map_err(|e| log_err(failure::Error::from(e), log::Level::Warn))
        .unwrap_or(false);
}

fn process_path(path: PathBuf, backups: &Vec<Backup>) {
    if let Some(res) = backups.par_iter()
        .map(|b| backup(&path, b))
        .reduce_with(consolidate) {
            if let Err(e) = res {
                log_err(e, log::Level::Warn);
            }
        }
}

fn backup(p: &Path, b: &Backup) -> Result<()> {
    match b.name.as_ref() {
        "github-gists" => github::upload(p, b),
        _ => custom::upload(p, b)
    }
}

fn consolidate(r1: Result<()>, r2: Result<()>) -> Result<()> {
    if let Err(e) = r1 {
        log_err(e, log::Level::Warn);
        return r2;
    }
    r1.and(r2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_modified() {
        let time = SystemTime::now();

        let result = has_been_modified(time, 10);

        assert!(result)
    }

    #[test]
    fn should_not_be_modified() {
        let time = SystemTime::now() - (Duration::from_secs(10));

        let result = has_been_modified(time, 5);

        assert!(!result);
    }
}
