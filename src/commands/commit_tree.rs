use std::env;

use chrono::Local;
use hex::encode;

use crate::{structures::{commit::Commit, repo::Repo}, utils::user_edit_file};

use anyhow::anyhow;

pub fn exec(tree: String, parent: Option<String>, message: Option<String>) -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    let author = "alon".to_string();
    let email = "alonlevshani@gmail.com".to_string();

    let timestamp = Local::now();

    let message = user_edit_file(&repo,"COMMITMSG","commit message")?;

    let commit = Commit::new(
        tree,
        parent,
        author.clone(),
        author,
        email,
        timestamp,
        Some(message),
    );

    let commit = commit.to_object();

    let hash = commit.write(&repo)?;

    println!("{}", encode(hash));

    Ok(())
}
