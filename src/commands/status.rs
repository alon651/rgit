use anyhow::{anyhow, bail};
use std::env;

use crate::structures::repo::Repo;

pub fn exec() -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    match repo.get_branch()? {
        Some(branch) => println!("on branch: {}", branch),
        None => bail!("didnt found a branch"),
    }
    Ok(())
}
