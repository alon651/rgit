use std::{env, path::PathBuf};

use anyhow::anyhow;

use crate::structures::repo::Repo;

pub fn exec(paths: Vec<PathBuf>) -> anyhow::Result<()> {
    let mut abs_paths = Vec::new();
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    for path in paths {
        let current_path = env::current_dir()?.join(path);
        abs_paths.push(current_path);
    }

    repo.remove_paths_from_index(&abs_paths)?;

    Ok(())
}
