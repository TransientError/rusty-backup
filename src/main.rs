extern crate serde_json;
extern crate serde;
extern crate reqwest;
extern crate rayon;

use backup::{appconfig, backup_performer, archive_generator};
use rayon::prelude::*;

const CONFIG_PATH: &str = "./backup.json";

fn main() {
    let config = appconfig::read_config(CONFIG_PATH);
    
    config.package_managers.par_iter().for_each(archive_generator::generate_archive);
    backup_performer::perform_backup(config.updaters);

    println!("hello")
}
