use anyhow::anyhow;
use std::{env, path::Path};

use crate::structures::{commit::Commit, object::Object, repo::Repo};

pub fn exec(commit: String, path: &Path) -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    let commit_obj = Object::read(&repo, &commit)?;
    let commit = Commit::from_object(&commit_obj)?;

    commit.unpack_at(&repo, path)
}
