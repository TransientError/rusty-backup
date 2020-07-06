use crate::appconfig::Backup;
use crate::log_err;
use crate::uploaders::{custom, github};
use anyhow::Result;

use anyhow::{bail, Error};
use futures::{
    channel::mpsc,
    future,
    stream::{self, StreamExt, TryStreamExt},
};
use log::info;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};
use std::result;
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

pub async fn perform_backup(backups: Vec<Backup>, path: &String) -> Result<()> {
    let ref_backups = Arc::new(backups);
    match fs::read_dir(path) {
        Ok(entries) => {
            stream::iter(entries.filter_map(process_file).filter_map(process_entry))
                .map(Result::Ok)
                .try_for_each(|p| {
                    let ref_backups = ref_backups.clone();
                    async move { process_path(p, ref_backups).await }
                })
                .await?;
        }
        Err(e) => return Err(Error::from(e)),
    }
    Ok(())
}

fn process_file(result: io::Result<DirEntry>) -> Option<DirEntry> {
    match result {
        Ok(dir) => Some(dir),
        Err(e) => {
            log_err(Error::from(e), log::Level::Warn);
            None
        }
    }
}

fn process_entry(entry: DirEntry) -> Option<PathBuf> {
    let one_day_secs = 86400;
    let modified_time = entry.metadata().and_then(|m| m.modified());

    match modified_time {
        result::Result::Ok(m) if has_been_modified(m, one_day_secs) => Some(entry.path()),
        result::Result::Ok(_) => {
            info!(
                "{} has not been modified; skipping",
                entry.file_name().to_string_lossy()
            );
            None
        }
        Err(e) => {
            log_err(Error::from(e), log::Level::Warn);
            None
        }
    }
}

fn has_been_modified(time: SystemTime, secs: u64) -> bool {
    return time
        .elapsed()
        .map(|dur| dur < Duration::from_secs(secs))
        .map_err(|e| log_err(Error::from(e), log::Level::Warn))
        .unwrap_or(false);
}

async fn process_path(path: PathBuf, backups: Arc<Vec<Backup>>) -> Result<()> {
    if let Err(e) = do_backup(path, backups).await {
        log_err(e, log::Level::Warn);
    }
    Ok(())
}

async fn do_backup(path: PathBuf, backups: Arc<Vec<Backup>>) -> Result<()> {
    let (mut tx_res, rx_res) = mpsc::channel(backups.len() + 2);
    let (tx_lim, rx_lim) = crossbeam::bounded(4);

    for _ in 0..4 {
        tx_lim.send(()).unwrap();
    }

    for b in backups.iter() {
        rx_lim.recv().map_err(Error::from).and_then(|_| {
            tx_res
                .try_send(tokio::spawn({
                    let path = path.clone();
                    let b = b.to_owned();
                    async move { backup(&path, &b).await }
                }))
                .map_err(Error::from)
        })?
    }

    let errs: Vec<Error> = rx_res
        .buffer_unordered(4)
        .filter_map(|res| future::ready(res.err().map(Error::from)))
        .collect()
        .await;

    if errs.is_empty() {
        Ok(())
    } else {
        bail!("{} backups failed because of e.g. {}", errs.len(), errs[0])
    }
}

async fn backup(p: &Path, b: &Backup) -> Result<()> {
    match b.name.as_ref() {
        "github-gists" => github::upload(p, b).await,
        _ => custom::upload(p, b),
    }
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
