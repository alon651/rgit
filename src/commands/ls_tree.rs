use crate::structures::{object::Object, repo::Repo, tree::Tree};

use anyhow::anyhow;
use std::env;

pub fn exec(hash: &str) -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;
    let obj = Object::read(&repo, hash)?;

    let tree = Tree::from_object(&obj)?;

    print!("{}", tree);

    Ok(())
}
