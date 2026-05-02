use anyhow::{Context, anyhow};
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};
use crate::structures::{commit::Commit, diff::Diff, object::Object, repo::Repo, tree::Tree};

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

    let index = repo.get_index().context("failed to read the index")?;

    if let Ok(commit) = &commit {
        let committed = flatten_committed_files(&repo, commit)?;

        let diff = Diff::from_index_and_repo(&index, &committed);

        print_sections(&[
            ("modified", &diff.modified),
            ("added", &diff.added),
            ("deleted", &diff.deleted),
        ]);

        println!(); // separator between staged and unstaged changes
    }

    let staged_diff = Diff::from_working_tree_and_index(&repo, &index)?;

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
