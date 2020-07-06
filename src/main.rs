use anyhow::{bail, Error, Result};
use appconfig::Archive;
use backup::{appconfig, archive_generator, backup_performer, logger};
use futures::{
    channel::mpsc,
    future,
    stream::StreamExt,
};
use log::{info, warn};
use mpsc::Sender;
use std::process;
use tokio::{runtime::Runtime, task::JoinHandle};

const CONFIG_PATH: &str = "/Users/kvwu/.config/backup/config.json";

fn main() {
    let mut rt = Runtime::new().unwrap();
    rt.block_on(async {
        if let Err(err) = run().await {
            log_err(err);
            process::exit(1);
        }
    })
}

async fn run() -> Result<()> {
    logger::init()?;
    let config = appconfig::read_config(CONFIG_PATH)?;

    if let Err(err) = generate_archives(config.archives, &config.archive_path).await {
        log_err(err);
    }

    backup_performer::perform_backup(config.backups, &config.archive_path).await?;

    Ok(info!("Done backing up"))
}

async fn generate_archives(archives: Vec<Archive>, archive_path: &String) -> Result<()> {
    let (mut tx_res, rx_res): (Sender<JoinHandle<_>>, _) = mpsc::channel(archives.len() + 2);
    let (tx_lim, rx_lim) = crossbeam::bounded(4);

    for _ in 0..4 {
        tx_lim.send(()).unwrap();
    }

    for archive in archives.into_iter() {
        match rx_lim.recv() {
            Ok(_) => tx_res
                .try_send({
                    let archive = archive.clone();
                    let archive_path = archive_path.clone();
                    let tx_lim = tx_lim.clone();
                    tokio::spawn(async move {
                        archive_generator::generate_archive(&archive, &archive_path).await.unwrap();
                        tx_lim.send(()).unwrap();
                    })
                })
                .unwrap(),
            Err(e) => bail!("{:#?}", e),
        }
    }

    let errs: Vec<Error> = rx_res
        .buffer_unordered(4)
        .filter_map(|res| future::ready(res.err().map(Error::from)))
        .collect()
        .await;

    if errs.is_empty() {
        Ok(())
    } else {
        bail!("{} archive generations failed because of e.g. {}", errs.len(), errs[0])
    }
}

fn log_err(e: Error) {
    warn!("{}", e);
    let backtrace = e.backtrace().to_string();
    if !backtrace.trim().is_empty() {
        warn!("{}", backtrace)
    }
}
