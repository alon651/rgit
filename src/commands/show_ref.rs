use std::{collections::HashMap, env, fs, path::Path};

use crate::structures::repo::Repo;
use anyhow::{anyhow, bail};
use std::collections::BTreeMap;

pub fn exec() -> anyhow::Result<()> {
    let repo =
        Repo::find(&std::env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;

    let refs_root = repo.data_dir.join("refs");

    if !refs_root.is_dir() {
        bail!("refs is not a directory");
    }

    let mut refs = BTreeMap::new();

    fill_refs(&repo, &refs_root, &refs_root, &mut refs)?;

    for (key, value) in refs {
        println!("{} {}", value, key);
    }

    Ok(())
}

fn fill_refs(
    repo: &Repo,
    current_path: &Path,
    base_path: &Path,
    map: &mut BTreeMap<String, String>,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(current_path)? {
        let path = entry?.path();

        if path.is_dir() {
            fill_refs(repo, &path, base_path, map)?;
        } else {
            if let Some(content) = repo.resolve_ref(&path, 10) {
                let display_path = path.strip_prefix(base_path)?.to_string_lossy().into_owned();

                map.insert(display_path, content);
            }
        }
    }
    Ok(())
}
