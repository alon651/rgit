use std::env;

use anyhow::anyhow;

use crate::{
    structures::{commit::Commit, object::Object, repo::Repo},
    utils::resolve_target_or_head,
};

pub fn exec(commit: Option<String>) -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    let mut current_commit = commit.or(Some(resolve_target_or_head(&repo, None)?));

    while let Some(ref hash) = current_commit {
        let obj = Object::read(&repo, hash, true)?;

        let commit_obj = Commit::from_object(&obj)?;
        let hash = obj.hash();

        println!("{}", commit_obj.pretty(hash));

        current_commit = commit_obj.parent
    }

    Ok(())
}
