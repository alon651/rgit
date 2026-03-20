use std::env;
use std::path::PathBuf;

use anyhow::Ok;

use crate::structures::repo::{DIR_NAME, Repo};

pub fn exec(path: Option<PathBuf>) -> anyhow::Result<()> {
    let path = path.unwrap_or_else(|| env::current_dir().unwrap());

    Repo::init(&path)?;

    println!("Initialized {} directory", DIR_NAME);

    Ok(())
}
