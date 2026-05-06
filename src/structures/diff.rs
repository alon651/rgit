use anyhow::Context;
use hex::encode;
use walkdir::WalkDir;

use crate::structures::{
    commit::Commit,
    index::Index,
    object::{Object, ObjectType},
    repo::Repo,
    tree::Tree,
};

use std::{
    collections::HashMap,
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Diff {
    pub modified: Vec<String>,
    pub added: Vec<String>,
    pub deleted: Vec<String>,
}

impl Diff {
    /// Diffs the working tree(current disk state) and the current index
    pub fn from_working_tree_and_index(repo: &Repo, index: &Index) -> anyhow::Result<Diff> {
        let mut modified = Vec::new();
        let mut added = Vec::new();
        let mut deleted = Vec::new();

        for entry in WalkDir::new(&repo.work_dir)
            .into_iter()
            .filter_entry(|e| !Repo::ignore(e.path()))
        {
            let entry = entry?;
            if !entry.path().is_file() {
                continue;
            }

            let relative = entry.path().strip_prefix(&repo.work_dir)?;
            let relative_str = relative.to_string_lossy().to_string();
            let metadata = entry.metadata()?;

            if !index.entries.contains_key(&relative_str) {
                added.push(relative_str);
            } else {
                let index_entry = &index.entries[&relative_str];
                if metadata.mtime() > index_entry.mtime as i64 {
                    if let Ok(true) = file_hash_changed(repo, relative, index_entry) {
                        modified.push(relative_str);
                    }
                }
            }
        }

        // check for files in the index that no longer exist on disk
        for path in index.entries.keys() {
            let relative = Path::new(&path);
            if !repo.work_dir.join(relative).exists() {
                deleted.push(path.clone());
            }
        }

        modified.sort();
        added.sort();
        deleted.sort();

        Ok(Diff {
            modified,
            added,
            deleted,
        })
    }

    /// Diffs the index and the saved tree in the repo
    pub fn from_index_and_repo(index: &Index, committed: &HashMap<PathBuf, String>) -> Diff {
        let mut modified = Vec::new();
        let mut added = Vec::new();
        let mut deleted = Vec::new();

        for (name, entry) in &index.entries {
            match committed.get(Path::new(name)) {
                Some(hash) if hash != &encode(entry.sha1) => modified.push(name.clone()),
                Some(_) => {}
                None => added.push(name.clone()),
            }
        }

        for name in committed.keys() {
            let name_str = name.to_string_lossy().to_string();
            if !index.entries.contains_key(&name_str) {
                deleted.push(name_str);
            }
        }

        modified.sort();
        added.sort();
        deleted.sort();

        Diff {
            modified,
            added,
            deleted,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.deleted.is_empty()
    }

    ///inverts a diff - converts the added to remove and the removed to added
    /// done to inverse diff direction
    pub fn inverse(&self) -> Diff {
        Diff {
            modified: self.modified.clone(),
            added: self.deleted.clone(),
            deleted: self.added.clone(),
        }
    }
}

fn file_hash_changed(
    repo: &Repo,
    relative: &Path,
    index_entry: &crate::structures::index::IndexEntry,
) -> anyhow::Result<bool> {
    let content = fs::read(repo.work_dir.join(relative)).context("failed to read object file")?;
    let object = Object::new(content, ObjectType::Blob);
    Ok(index_entry.sha1 != object.hash())
}

pub fn flatten_committed_files(
    repo: &Repo,
    commit: &str,
) -> anyhow::Result<HashMap<PathBuf, String>> {
    let commit_obj = Object::read(repo, commit, true)?;
    let commit = Commit::from_object(&commit_obj)?;

    let obj = Object::read(repo, &commit.tree, true)?;
    let tree = Tree::from_object(&obj)?;
    tree.flatten(repo, None)
}
