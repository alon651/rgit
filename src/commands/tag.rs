use std::fs;

use crate::{
    structures::{object::Object, repo::Repo, tag::Tag},
    utils::{resolve_target_or_head, user_edit_file},
};

use anyhow::{Ok, anyhow};
use chrono::Local;
use hex::encode;

pub fn exec(name: Option<String>, object: Option<String>, annonate: bool) -> anyhow::Result<()> {
    let repo =
        Repo::find(&std::env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    fs::create_dir_all(repo.data_dir.join("refs/tags"))?;

    match name {
        Some(name) => {
            if annonate {
                complex_tag(&repo, name, object)?
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

    let mut count = 0;
    for entry in tags_dir {
        count += 1;
        let entry = entry?;
        let name = entry.file_name();
        println!("{}", name.display())
    }
    if count == 0 {
        println!("didnt found any refs")
    }
    Ok(())
}

pub fn simple_tag(repo: &Repo, name: String, object: Option<String>) -> anyhow::Result<()> {
    let ref_dest = resolve_target_or_head(repo, object)?;

    let tag_path = repo.data_dir.join("refs/tags").join(name);

    fs::write(tag_path, ref_dest)?;

    Ok(())
}

pub fn complex_tag(repo: &Repo, name: String, object: Option<String>) -> anyhow::Result<()> {
    let message = user_edit_file(repo, "TAGANNT", "tag annotation")?;

    let tagger = "alon".to_string();
    let tagger_email = "alonlevshani@gmail.com".to_string();

    let timestamp = Local::now();

    let ref_dest = resolve_target_or_head(repo, object)?;

    let object_type = Object::read(repo, &ref_dest)?.object_type;

    let tag = Tag::new(
        &ref_dest,
        object_type,
        &name,
        &tagger,
        &tagger_email,
        timestamp,
        Some(message),
    );

    let tag_hash = tag.to_object().write(repo)?;

    let tag_hash = encode(tag_hash);

    simple_tag(repo, name, Some(tag_hash))?;

    Ok(())
}
