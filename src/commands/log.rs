use std::env;

use anyhow::{anyhow, bail};

use crate::structures::{commit::Commit, object::Object, repo::Repo};

pub fn exec(commit: Option<String>) -> anyhow::Result<()> {
    if commit.is_none() {
        bail!("logging the head is not yet implemented")
    };

    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    let mut current_commit = commit;

    while let Some(ref hash) = current_commit {
        let obj = Object::read(&repo, hash, true)?;

        let commit_obj = Commit::from_object(&obj)?;
        let hash = obj.hash();

        println!("{}", commit_obj.pretty(hash));

        current_commit = commit_obj.parent
    }

    Ok(())
}
