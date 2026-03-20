use anyhow::{Context, Ok, anyhow, bail};
use hex::encode;
use std::{env, fs, path::PathBuf};

use crate::structures::{
    object::{Object, ObjectType},
    repo::Repo,
};

pub fn exec(path: PathBuf, write: bool) -> anyhow::Result<()> {
    if path.is_dir() {
        bail!("unable to hash {}", path.to_str().unwrap())
    }

    let hash: String = if write {
        let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;
        encode(Object::write_blob_from_file(&repo, &path)?)
    } else {
        let content = fs::read(&path).context("failed to read object file")?;
        let object = Object::new(content, ObjectType::Blob);
        encode(object.hash())
    };

    println!("{}", hash);

    Ok(())
}
