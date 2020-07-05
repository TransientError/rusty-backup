#![cfg_attr(test, feature(proc_macro_hygiene))]

#[macro_use]
extern crate serde_derive;

use anyhow::Error;
use log::log;

pub mod appconfig;
pub mod archive_generator;
pub mod backup_performer;
pub mod logger;
pub mod uploaders;

pub fn log_err(e: Error, l: log::Level) {
    log!(l, "{}", e);
    let backtrace = e.backtrace().to_string();
    if !backtrace.trim().is_empty() {
        log!(l, "{}", backtrace)
    }
}
