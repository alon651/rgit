use std::{collections::HashMap, env, path::Path};

use chrono::Local;
use hex::encode;

use crate::{structures::repo::Repo, utils::user_edit_file};
use anyhow::{Ok, anyhow};

pub fn exec(message: Option<String>) -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    let author = "alon".to_string();
    let email = "alonlevshani@gmail.com".to_string();

    let timestamp = Local::now();

    let message = match message {
        Some(message) => message,
        None => user_edit_file(&repo, "COMMITMSG", "commit message")?,
    };

    // println!("{}",hm)
    tree_from_index(&repo)?;

    Ok(())
}

pub fn tree_from_index(repo: &Repo) -> anyhow::Result<()> {
    let index = repo.get_index()?;

    let mut hm = HashMap::new();

    for (path, entry) in index.entries {
        let entry_parent = Path::new(&path).parent();
        let Some(entry_parent) = entry_parent.clone() else { continue };
        // hm.insert(entry_parent.to_path_buf(), entry);
        hm.entry(entry_parent.to_path_buf()).or_insert(Vec::new()).push(entry);
    }

    let mut keys: Vec<_> = hm.keys().collect();

    keys.sort_by_key(|k| k.display().to_string().len());

    keys.reverse();
    // hm[&repo.work_dir].


    keys.iter().for_each(|a| {
        for entry in &hm[*a] {
            println!("{} {} {}", entry.mode, encode(entry.sha1), entry.name);
        }
    });

    // Ok(tree)

    Ok(())
}
