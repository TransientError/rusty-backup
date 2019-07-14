#[macro_use]
extern crate serde_derive;

use log::log;
use std::result;
type Result<T> = result::Result<T, failure::Error>;

pub mod appconfig;
pub mod archive_generator;
pub mod backup_performer;
pub mod uploaders;
pub mod logger;

pub fn log_err(e: failure::Error, l: log::Level) {
    log!(l, "{}", e);
    let backtrace = e.backtrace().to_string();
        if !backtrace.trim().is_empty() {
            log!(l, "{}", backtrace)
        }
}