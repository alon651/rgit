use std::env;

use crate::structures::repo::Repo;

use anyhow::anyhow;

pub fn exec() -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    let main_content = repo.resolve_ref("HEAD").unwrap();

    println!("{main_content}");

    Ok(())
}
