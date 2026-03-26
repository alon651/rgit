use std::{env, fs, process::Command};

use crate::structures::repo::Repo;

pub fn user_edit_file(repo:&Repo,filename:&str, content_name: &str) -> anyhow::Result<String> {
    let path = repo.data_dir.join(filename);

    fs::write(&path, format!("\n# Please enter your {} here\n# and then quit(for vim hit escape then :wq and enter) \n",content_name))?;

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


fn parse_file_output(content:&str)->String{
    content.lines().filter(|line| !line.trim().starts_with("#")).collect::<Vec<_>>().join("\n").trim().to_string()
}