use std::{env, fs, path::Path, process::Command};

use crate::structures::{object::Object, repo::Repo};

use anyhow::{Context, anyhow};

pub fn user_edit_file(repo: &Repo, filename: &str, content_name: &str) -> anyhow::Result<String> {
    let path = repo.data_dir.join(filename);

    fs::write(
        &path,
        format!(
            "\n# Please enter your {} here\n# and then quit(for vim hit escape then :wq and enter) \n",
            content_name
        ),
    )?;

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    let status = Command::new(editor).arg(&path).status()?;

    let mut content = String::new();

    if status.success() {
        content = std::fs::read_to_string(&path)?;
        fs::remove_file(path)?;
    } else {
        eprintln!("Editor exited with error");
    }

    Ok(parse_file_output(&content))
}

fn parse_file_output(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.trim().starts_with("#"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Resolves an object identifier or defaults to the current HEAD.
///
/// If `object` is `Some`, it returns the identifier as resolved using the `Object::find_object` function.
/// If `None`, it attempts to resolve the reference pointed to by HEAD.
///
/// # Errors
/// Returns an error if HEAD cannot be resolved (e.g., in an empty repository).
pub fn resolve_target_or_head(repo: &Repo, object: Option<String>) -> anyhow::Result<String> {
    match object {
        Some(obj) => {
            let path = Object::find_object(repo, &obj)?;

            let file_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow!("Invalid file: {}", path.display()))?;

            let dir = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow!("Invalid directory: {}", path.display()))?;

            Ok(format!("{}{}", dir, file_name))
        }
        None => {
            let head = repo.get_head()?;
            repo.resolve_ref(Path::new(&head), 10).context(
                "Could not resolve HEAD reference. Ensure the repository has at least one commit.",
            )
        }
    }
}
