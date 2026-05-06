use crate::{
    structures::{
        commit::unpack_blob,
        diff::{Diff, flatten_committed_files},
        object::Object,
        repo::Repo,
    },
    utils::resolve_target_or_head,
};
use anyhow::{Context, anyhow, ensure};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

pub fn exec(target: &str, create_branch: bool) -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;
    let is_clean = repo
        .is_working_tree_clean()
        .context("checking working tree safety")?;

    ensure!(is_clean, "working tree is dirty");

    if create_branch {
        let hash = resolve_target_or_head(&repo, None)?;
        repo.upsert_branch(target, &hash)?;
        repo.move_to_branch(target)?;
    } else {
        let index = repo.get_index()?;

        // delete all files in index

        let committed = flatten_committed_files(&repo, target)?;

        let diff = Diff::from_index_and_repo(&index, &committed).inverse();

        apply_diff(&repo, &committed, &diff)?;

        if let Some(path) = Object::is_refrence(&repo, target) {
            repo.write_to_head(&format!("ref: {}", path.display()))?;
        } else {
            let full_name = Object::expand_object_hash(&repo, target)?;
            ensure!(full_name.is_some(), "cannot find target: {}", target);

            let full_name = full_name.unwrap();

            //assemble hash
            let hash = format!(
                "{}{}",
                full_name.parent().unwrap().file_name().unwrap().display(),
                full_name.file_name().unwrap().display(),
            );

            repo.write_to_head(&hash)?;
        }

        //todo! sync index
    }

    Ok(())
}

fn apply_diff(
    repo: &Repo,
    committed: &HashMap<PathBuf, String>,
    diff: &Diff,
) -> anyhow::Result<()> {
    //modify:
    for file in &diff.modified {
        let path = Path::new(file);
        let hash = committed.get(path).unwrap();
        println!("modified file: {}", file);
        unpack_blob(repo, path, hash)?;
    }

    //added:
    for file in &diff.added {
        let path = Path::new(file);
        let hash = committed.get(path).unwrap();
        fs::create_dir_all(path.parent().unwrap())?;
        println!("created file: {}", file);
        unpack_blob(repo, path, hash)?;
    }

    //deleted:
    for file in &diff.deleted {
        let path = Path::new(file);
        println!("deleted file: {}", file);
        fs::remove_file(path)?;
    }

    Ok(())
}
