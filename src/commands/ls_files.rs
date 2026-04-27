use std::env;

use anyhow::{Context, anyhow};
use hex::encode;

use crate::structures::repo::Repo;

pub fn exec() -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;
    let index = repo.get_index().context("error on getting index of repo")?;
    for (name, _entry) in index.entries {
        println!("{} {}", name, encode(_entry.sha1))
    }
    Ok(())
}
