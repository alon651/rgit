use std::fs;

use crate::{structures::repo::Repo, utils::resolve_target_or_head};
use anyhow::anyhow;
use walkdir::WalkDir;

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
    for entry in WalkDir::new(&branches_dir_path) {
        let entry = entry?;
        if entry.path().is_file() {
            let branch_name = entry.path().strip_prefix(&branches_dir_path)?;
            println!("{}", branch_name.display())
        }
    }
    Ok(())
}
