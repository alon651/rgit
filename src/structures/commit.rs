use crate::structures::{
    object::{Object, ObjectType},
    repo::Repo,
    tree::Tree,
};
use anyhow::{Context, bail, ensure};
use chrono::{DateTime, Local};
use colored::Colorize;
use hex::encode;
use std::{fmt, fs, path::Path};

/// A Git-style commit object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub tree: String,
    pub parent: Option<String>,
    pub author: String,
    pub committer: String,
    pub email: String,
    pub timestamp: DateTime<Local>,
    pub message: Option<String>,
}

impl Commit {
    pub fn new(
        tree: String,
        parent: Option<String>,
        author: String,
        committer: String,
        email: String,
        timestamp: DateTime<Local>,
        message: Option<String>,
    ) -> Self {
        Self {
            tree,
            parent,
            author,
            committer,
            email,
            timestamp,
            message,
        }
    }

    pub fn to_object(&self) -> Object {
        Object::new(self.to_string().into_bytes(), ObjectType::Commit)
    }

    pub fn from_object(object: &Object) -> anyhow::Result<Self> {
        ensure!(object.object_type == ObjectType::Commit, "Object must be Commit");


        let (headers, message) = Object::parse_key_value(&object.data)?;

        // get the first value of a header or error
        let get_required = |key| {
            headers
                .get(key)
                .and_then(|v| v.first())
                .ok_or_else(|| anyhow::anyhow!("missing {} in commit", key))
        };

        let author_line = get_required("author")?;

        let (author, email, timestamp) = parse_user_line(author_line, "author")?;

        Ok(Self {
            tree: get_required("tree")?.clone(),
            // parents: headers.get("parent").cloned().unwrap_or_default(),
            parent: get_required("parent").ok().cloned(),
            committer: parse_user_line(get_required("committer")?, "committer")?.0,
            author,
            email,
            timestamp,
            message,
        })
    }

    pub fn pretty(&self, hash: [u8; 20]) -> String {
        let title = format!("commit {}", encode(hash));

        let message = self.message.clone().unwrap_or("".to_string());

        format!(
            "{}\nAuthor: {} <{}>\nDate:   {}\n\n    {}\n",
            title.yellow(),
            self.author,
            self.email,
            self.timestamp.to_rfc2822(),
            message
        )
    }

    pub fn unpack_at(&self, repo: &Repo, path: &Path) -> anyhow::Result<()> {
        unpack_tree(repo, &self.tree, path)
    }
}

fn unpack_tree(repo: &Repo, tree_hash: &str, path: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(path)?;

    let tree_obj = Object::read(repo, tree_hash)?;
    let tree = Tree::from_object(&tree_obj)?;

    for entry in tree.entries {
        match entry.mode {
            0o100644 | 0o100755 | 0o120000 => {
                unpack_blob(repo, &path.join(entry.name), entry.hash)?;
            }
            0o40000 => {
                unpack_tree(repo, &encode(entry.hash), &path.join(entry.name))?;
            }
            mode => bail!("invalid mode {}", mode),
        }
    }

    Ok(())
}

fn unpack_blob(repo: &Repo, path: &Path, hash: [u8; 20]) -> anyhow::Result<()> {
    let obj = Object::read(repo, &encode(hash))?;

    fs::write(path, obj.data)?;

    Ok(())
}

fn parse_user_line(
    author_line: &str,
    arg: &str,
) -> anyhow::Result<(String, String, DateTime<Local>)> {
    let parts: Vec<&str> = author_line.splitn(3, ' ').collect();
    if parts.len() != 3 {
        anyhow::bail!("invalid {} line format in commit object", arg);
    }

    let timestamp_str = parts[2];

    let email = parts[1].trim_matches(|c| c == '<' || c == '>');
    let name = parts[0];
    let timestamp = DateTime::parse_from_str(timestamp_str, "%s %z")
        .context(format!("invalid timestamp: {} in {}", timestamp_str, arg))?
        .with_timezone(&Local);

    Ok((name.to_string(), email.to_string(), timestamp))
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut body = String::new();

        body.push_str(&format!("tree {}\n", self.tree));

        if let Some(parent) = &self.parent {
            body.push_str(&format!("parent {}\n", parent));
        }

        let ts = self.timestamp.format("%s %z");
        body.push_str(&format!("author {} <{}> {}\n", self.author, self.email, ts));
        body.push_str(&format!(
            "committer {} <{}> {}\n\n",
            self.committer, self.email, ts
        ));

        if let Some(message) = &self.message {
            body.push_str(&format!("{}\n", message));
        }

        write!(f, "{}", body)
    }
}
