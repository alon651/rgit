use crate::structures::index::Index;
use anyhow::{Ok, bail};
use std::{
    fs::{self},
    path::{Path, PathBuf},
};

pub const DIR_NAME: &str = ".rgit";

#[derive(Debug)]
pub struct Repo {
    pub work_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Repo {
    fn new(path: &Path) -> Self {
        Repo {
            work_dir: path.to_path_buf(),
            data_dir: path.join(DIR_NAME),
        }
    }

    pub fn init(path: &Path) -> anyhow::Result<Self> {
        let repo = Self::new(path);

        if repo.data_dir.exists() {
            anyhow::bail!("repository already exists in {:?}", repo.work_dir)
        }

        fs::create_dir_all(&repo.data_dir)?;
        fs::create_dir_all(repo.data_dir.join("objects"))?;
        fs::create_dir_all(repo.data_dir.join("refs/heads"))?;

        fs::write(repo.data_dir.join("HEAD"), "ref: refs/heads/main\n")?;

        Ok(repo)
    }

    ///recursive check for the data directory
    pub fn find(path: &Path) -> Option<Self> {
        let mut path = path.to_path_buf();

        loop {
            let data_dir = path.join(DIR_NAME);
            if data_dir.is_dir() {
                return Some(Self {
                    work_dir: path,
                    data_dir,
                });
            } else if !path.pop() {
                break;
            }
        }
        None
    }

    ///if to ignore the file
    pub fn ignore(path: &Path) -> bool {
        let file_name = path.file_name().and_then(|s| s.to_str());
        matches!(
            file_name,
            Some(".git") | Some(".rgit") | Some("target") | Some("node_modules")
        )
    }

    pub fn resolve_ref(&self, path: &Path, depth: usize) -> Option<String> {
        if depth == 0 {
            return None;
        }

        let ref_path = self.data_dir.join(path);
        let content = fs::read_to_string(ref_path).ok()?;
        let trimmed = content.trim();

        if let Some(next_ref) = trimmed.strip_prefix("ref: ") {
            self.resolve_ref(Path::new(next_ref), depth - 1)
        } else {
            Some(trimmed.to_string())
        }
    }

    /// Get the head content stripped from the prefix
    pub fn get_head(&self) -> anyhow::Result<String> {
        let path = self.data_dir.join("HEAD");

        let content = fs::read_to_string(path)?;

        let stripped = content.trim().strip_prefix("ref: ");

        match stripped {
            Some(ref_path) => Ok(ref_path.to_string()),
            None => bail!("HEAD file is corrupted"),
        }
    }

    /// Get the path of a tag by its name
    /// Returns an error if the tag doesn't exist
    pub fn get_tag_path(&self, name: &str) -> anyhow::Result<PathBuf> {
        if name.contains('/') || name.contains('\\') || name.contains("..") || name.is_empty() {
            bail!("invalid tag name: {}", name);
        }
        let path = self.data_dir.join("refs").join("tags").join(name);
        if path.is_file() {
            Ok(path)
        } else {
            bail!("tag not found: {}", name);
        }
    }

    pub fn upsert_branch(&self, branch: &str, hash: &str) -> anyhow::Result<()> {
        let branch_path = self.data_dir.join("refs/heads").join(branch);

        if let Some(parent) = Path::new(&branch_path).parent() {
            //handle cases where the branch-name have slashes like feat/implement_feature
            fs::create_dir_all(parent)?;
        }

        fs::write(branch_path, hash)?;
        Ok(())
    }

    pub fn get_index(&self) -> anyhow::Result<Index> {
        Index::read(&self.data_dir.join("index"))
    }
}
