use crate::structures::repo::Repo;
use anyhow::{Context, anyhow, ensure};
use std::env;

pub fn exec(target: &str, create_branch: bool) -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;
    let is_clean = repo.is_working_tree_clean().context("checking working tree safety")?;

    ensure!(is_clean,"working tree is dirty");

    

    Ok(())
}
