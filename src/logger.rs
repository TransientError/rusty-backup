use std::result;
use log::{self, Log};
use chrono::prelude::*;

use crate::Result;

pub fn init() -> Result<()> {
    Ok(Logger::init().unwrap())
}

#[derive(Debug)]
struct Logger();

const LOGGER: &'static Logger = &Logger();

impl Logger {
    fn init() -> result::Result<(), log::SetLoggerError> {
        log::set_logger(LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Info))
    }
}

impl Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        eprintln!("{} {}: {}", Local::now().format("%Y-%m-%dT%H:%M:%S"), record.level(), record.args())
    }

    fn flush(&self) {
        // We use eprintln! which is flushed on every call.
    }
}
