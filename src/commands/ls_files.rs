use std::env;

use anyhow::{Context, anyhow};

use crate::structures::repo::Repo;

pub fn exec() -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;
    let index = repo.get_index().context("error on getting index of repo")?;
    for entry in index.entries {
        println!("{}", entry.name)
    }
    Ok(())
}
