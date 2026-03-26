use std::{fs, path::Path};

use crate::{structures::repo::Repo, utils::user_edit_file};

use anyhow::{Context, Ok, anyhow, bail};

pub fn exec(name: Option<String>, object: Option<String>, annonate: bool) -> anyhow::Result<()> {
    let repo =
        Repo::find(&std::env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    fs::create_dir_all(repo.data_dir.join("refs/tags"))?;

    match name {
        Some(name) => {
            if annonate {
                let message = user_edit_file(&repo,"TAGANNT","tag annotation")?;
                
            } else {
                simple_tag(&repo, name, object)?
            }
        }
        None => list_tags(&repo)?,
    }

    Ok(())
}

pub fn list_tags(repo: &Repo) -> anyhow::Result<()> {
    let tags_dir = fs::read_dir(repo.data_dir.join("refs/tags"))?;

    for entry in tags_dir {
        let entry = entry?;
        let name = entry.file_name();
        println!("{}", name.display())
    }

    Ok(())
}

pub fn simple_tag(repo: &Repo, name: String, object: Option<String>) -> anyhow::Result<()> {
    let ref_dest = match object {
        Some(object_hash) => object_hash,
        None => {
            let head = repo.get_head()?;
            let path = Path::new(&head);
            repo.resolve_ref(path, 10).context(
                "could not resolve the refrence of head, maybe you didnt commit anything yet?",
            )?
        }
    };

    let tag_path = repo.data_dir.join("refs/tags").join(name);

    fs::write(tag_path, ref_dest)?;

    Ok(())
}
