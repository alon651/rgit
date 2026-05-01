use anyhow::{Context, anyhow};
use hex::encode;
use std::{
    collections::HashMap,
    env, fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::structures::{
    commit::Commit,
    index::Index,
    object::{Object, ObjectType},
    repo::Repo,
    tree::Tree,
};

struct Diff {
    modified: Vec<String>,
    added: Vec<String>,
    deleted: Vec<String>,
}

pub fn exec() -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    let branch = repo.get_branch()?;

    let commit = repo
        .resolve_ref(Path::new(&"HEAD"), 10)
        .context("failed to resolve commit from branch");

    match branch {
        Some(branch) => println!("repo at branch {}", branch),
        None => {
            if let Ok(commit) = &commit {
                println!("HEAD Detached at commit {}", commit);
            } else {
                println!()
            }
        }
    };
    println!();

    let index = repo.get_index().context("failed to read the index")?;

    if let Ok(commit) = &commit {
        let committed = flatten_committed_files(&repo, commit)?;

        let diff = diff_index_with_tree(&index, &committed);

        print_sections(&[
            ("modified", &diff.modified),
            ("added", &diff.added),
            ("deleted", &diff.deleted),
        ]);

        println!(); // separator between staged and unstaged changes
    }

    let staged_diff = dif_disk_with_index(&repo, &index)?;

    print_sections(&[
        ("modified but not staged", &staged_diff.modified),
        ("added but not staged", &staged_diff.added),
        ("deleted but not staged", &staged_diff.deleted),
    ]);

    Ok(())
}

fn flatten_committed_files(repo: &Repo, commit: &str) -> anyhow::Result<HashMap<PathBuf, String>> {
    let commit_obj = Object::read(repo, commit, true)?;
    let commit = Commit::from_object(&commit_obj)?;

    let obj = Object::read(repo, &commit.tree, true)?;
    let tree = Tree::from_object(&obj)?;
    tree.flatten(repo, None)
}

fn diff_index_with_tree(index: &Index, committed: &HashMap<PathBuf, String>) -> Diff {
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

fn print_sections(sections: &[(&str, &[String])]) {
    let visible: Vec<_> = sections
        .iter()
        .filter(|(_, items)| !items.is_empty())
        .collect();

    for (i, (label, items)) in visible.iter().enumerate() {
        if i > 0 {
            println!();
        }
        for item in *items {
            println!("  {}: {}", label, item);
        }
    }
}

fn dif_disk_with_index(repo: &Repo, index: &Index) -> anyhow::Result<Diff> {
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

fn file_hash_changed(
    repo: &Repo,
    relative: &Path,
    index_entry: &crate::structures::index::IndexEntry,
) -> anyhow::Result<bool> {
    let content = fs::read(repo.work_dir.join(relative)).context("failed to read object file")?;
    let object = Object::new(content, ObjectType::Blob);
    Ok(index_entry.sha1 != object.hash())
}
