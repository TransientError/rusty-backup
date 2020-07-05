use backup::{appconfig, backup_performer, archive_generator, logger};
use rayon::prelude::*;
use std::result;
use std::process;
use log::{info, warn};

type Result<T> = result::Result<T, failure::Error>;

const CONFIG_PATH: &str = "/Users/kvwu/.config/backup/config.json";

fn main() {
    if let Err(err) = run() {
        log_err(err);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    logger::init()?;
    let config = appconfig::read_config(CONFIG_PATH)?;
    
    if let Some(res) = config.archives.par_iter()
        .map(|archive| archive_generator::generate_archive(archive, &config.archive_path))
        .reduce_with(consolidate) {
            if let Err(e) = res {
                log_err(e)
            }
        }
    
    backup_performer::perform_backup(config.backups, &config.archive_path);

    Ok(info!("Done backing up"))
}

fn consolidate(r1: Result<()>, r2: Result<()>) -> Result<()> {
    if let Err(e) = r1 {
        log_err(e);
        return r2;
    }
    r1.and(r2)
}

fn log_err(e: failure::Error) {
    warn!("{}", e);
    let backtrace = e.backtrace().to_string();
        if !backtrace.trim().is_empty() {
            warn!("{}", backtrace)
        }
}
