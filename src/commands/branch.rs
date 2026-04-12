use std::fs;

use crate::{structures::repo::Repo, utils::resolve_target_or_head};
use anyhow::anyhow;

//TODO: implement speerated branches- so a branch called feat/implement_thing will be saved in .rgit/refs/heads/feat/implement_thing
pub fn exec(name: Option<String>) -> anyhow::Result<()> {
    let repo =
        Repo::find(&std::env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    fs::create_dir_all(repo.data_dir.join("refs/tags"))?;

    match name {
        Some(name) => {
            let hash = resolve_target_or_head(&repo, None)?;
            repo.upsert_branch(&name, &hash)?;
        }
        None => {
            list_branches(&repo)?;
        }
    }

    Ok(())
}

fn list_branches(repo: &Repo) -> anyhow::Result<()> {
    let branches_dir_path = repo.data_dir.join("refs/heads");
    let branches_dir = fs::read_dir(branches_dir_path)?;
    for entry in branches_dir {
        let entry = entry?;
        let name = entry.file_name();
        println!("{}", name.display())
    }
    Ok(())
}
