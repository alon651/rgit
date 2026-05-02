use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use chrono::Local;
use hex::encode;

use crate::{
    structures::{
        commit::Commit,
        repo::Repo,
        tree::{Tree, TreeEntry},
    },
    utils::{resolve_target_or_head, user_edit_file},
};

use anyhow::{Ok, anyhow, bail};

pub fn exec(message: Option<String>) -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    let author = "alon".to_string();
    let email = "alonlevshani@gmail.com".to_string();

    let timestamp = Local::now();

    let message = match message {
        Some(message) => message,
        None => user_edit_file(&repo, "COMMITMSG", "commit message")?,
    };

    let root_sha = tree_from_index(&repo)?;
    let parent = resolve_target_or_head(&repo, None).ok();
    let commit = Commit::new(
        encode(root_sha),
        parent,
        author.clone(),
        author,
        email,
        timestamp,
        Some(message),
    );

    let hash = commit.to_object().write(&repo)?;

    let hash_str = encode(hash);

    if repo.is_currently_at_branch() {
        let branch_path = repo.get_head()?;
        fs::write(repo.data_dir.join(branch_path), hash_str)?;
    } else {
        bail!("writing commit to a detached head not supported")
    }

    Ok(())
}

pub fn tree_from_index(repo: &Repo) -> anyhow::Result<[u8; 20]> {
    let index = repo.get_index()?;

    // list of file index entries directly in that dir
    let mut files_by_dir: HashMap<PathBuf, Vec<_>> = HashMap::new();
    // list of immediate sub-directory paths
    let mut subdirs_by_dir: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

    for (path, entry) in index.entries {
        let path = PathBuf::from(&path);
        let Some(parent) = path.parent().map(|p| p.to_path_buf()) else {
            continue;
        };

        files_by_dir.entry(parent.clone()).or_default().push(entry);

        // walk every folder until root(local root)
        let mut child = parent;
        loop {
            let Some(grand) = child.parent().map(|p| p.to_path_buf()) else {
                break;
            };
            // ensure each ancestor exists in maps
            files_by_dir.entry(grand.clone()).or_default();
            let subs = subdirs_by_dir.entry(grand.clone()).or_default();
            if !subs.contains(&child) {
                subs.push(child.clone());
            }
            if grand.as_os_str().is_empty() {
                break;
            }
            child = grand;
        }
    }

    // sort dir keys longest first
    let mut keys: Vec<PathBuf> = files_by_dir.keys().cloned().collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.components().count()));

    // from dir to sha of dir
    let mut dir_shas: HashMap<PathBuf, [u8; 20]> = HashMap::new();

    for key in &keys {
        let mut entries: Vec<TreeEntry> = Vec::new();

        // file entries directly under this dir
        for ie in &files_by_dir[key] {
            let name = Path::new(&ie.name)
                .file_name()
                .ok_or_else(|| anyhow!("index entry has no file name: {}", ie.name))?
                .to_string_lossy()
                .into_owned();

            entries.push(TreeEntry {
                mode: 0o100644,
                name,
                hash: ie.sha1,
            });
        }

        // sub-directories under this dir
        if let Some(subs) = subdirs_by_dir.get(key) {
            for sub in subs {
                let sha = dir_shas
                    .get(sub)
                    .copied()
                    .ok_or_else(|| anyhow!("missing sub-tree sha for {:?}", sub))?;
                let name = sub
                    .file_name()
                    .ok_or_else(|| anyhow!("sub-dir has no file name: {:?}", sub))?
                    .to_string_lossy()
                    .into_owned();

                entries.push(TreeEntry {
                    mode: 0o40000,
                    name,
                    hash: sha,
                });
            }
        }

        let tree = Tree::new(entries);
        let sha = tree.write(repo)?;
        dir_shas.insert(key.clone(), sha);
    }

    let root = PathBuf::new();
    dir_shas
        .get(&root)
        .copied()
        .ok_or_else(|| anyhow!("no root tree was built (empty index?)"))
}
