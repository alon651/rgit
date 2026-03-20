use std::{env, fs, path::PathBuf};

use anyhow::{Ok, anyhow};
use hex::encode;

use crate::structures::{
    object::Object,
    repo::Repo,
    tree::{Tree, TreeEntry},
};

pub fn exec() -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    let hash = write_tree(&repo, &repo.work_dir)?;

    let hash = encode(hash);

    println!("{hash}");

    Ok(())
}

fn write_tree(repo: &Repo, path: &PathBuf) -> anyhow::Result<[u8; 20]> {
    if path.is_file() {
        return Object::write_blob_from_file(repo, path);
    }

    write_directory_tree(repo, path)
}

fn write_directory_tree(repo: &Repo, path: &PathBuf) -> anyhow::Result<[u8; 20]> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        if let Some(tree_entry) = process_dir_entry(repo, entry?) {
            entries.push(tree_entry);
        }
    }

    let tree = Tree::new(entries);
    let hash = tree.write(repo)?;

    Ok(hash)
}

fn process_dir_entry(repo: &Repo, entry: fs::DirEntry) -> Option<TreeEntry> {
    let path = entry.path();
    let name = entry.file_name().to_str()?.to_string();
    let mode = if path.is_dir() { 0o40000 } else { 0o100644 };

    if is_empty_dir(&path).ok()? {
        return None;
    }

    if Repo::ignore(&path) {
        return None;
    }

    let hash = write_tree(repo, &path).ok()?;

    Some(TreeEntry { mode, name, hash })
}

fn is_empty_dir(path: &PathBuf) -> anyhow::Result<bool> {
    Ok(path.is_dir() && fs::read_dir(path)?.next().is_none())
}
