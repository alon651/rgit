use crate::{
    structures::{commit::Commit, index::Index, object::Object},
    utils::get_children_of_dir,
};
use anyhow::{Context, bail};
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

    pub fn index_path(&self) -> PathBuf {
        self.data_dir.join("index")
    }

    pub fn get_index(&self) -> anyhow::Result<Index> {
        let index_path = self.index_path();
        if index_path.is_file() {
            let index = Index::read(&self.index_path()).context("index file is broken")?;
            Ok(index)
        } else {
            let index = Index::new();
            Ok(index)
        }
    }

    pub fn save_index(&self, index: Index) -> anyhow::Result<()> {
        index.save_index(&self.index_path())
    }

    /// Add files to the repo index
    pub fn add_paths_to_index(&self, paths: &[PathBuf]) -> anyhow::Result<()> {
        let mut index = self.get_index()?;
        self.index_pathspecs(paths, &mut index)?;
        self.save_index(index)?;
        Ok(())
    }

    fn index_pathspecs(&self, paths: &[PathBuf], index: &mut Index) -> anyhow::Result<()> {
        for path in paths {
            if Self::ignore(path) {
                continue;
            }

            if path.is_dir() {
                let children = get_children_of_dir(path)?;
                self.index_pathspecs(&children, index)?;
            } else if path.is_file() || !self.has_tracked_children(path, index) {
                // index_pathspec would error for deleted dirs with tracked children, will only reach this path for deleted dirs
                // example: if src deleted and has tracked childs the remove stale entry will remove them, if not it will do nothing
                self.index_pathspec(path, index)?;
            }

            self.remove_stale_entries(path, index);
        }

        Ok(())
    }

    fn has_tracked_children(&self, path: &Path, index: &Index) -> bool {
        let Ok(rel) = path.strip_prefix(&self.work_dir) else {
            return false;
        };
        let rel = rel.to_string_lossy();
        if rel.is_empty() {
            return !index.entries.is_empty();
        }
        let prefix = format!("{}/", rel.trim_end_matches('/'));
        index.entries.keys().any(|k| k.starts_with(&prefix))
    }

    fn remove_stale_entries(&self, path: &Path, index: &mut Index) {
        // drop index entries whose files deleted from disk.
        let Ok(rel) = path.strip_prefix(&self.work_dir) else {
            return;
        };
        let rel = rel.to_string_lossy();

        let prefix = if rel.is_empty() {
            String::new()
        } else {
            format!("{}/", rel.trim_end_matches('/'))
        };

        let to_remove: Vec<String> = index
            .entries
            .keys()
            .filter(|k| k.starts_with(&prefix) && !self.work_dir.join(k).exists())
            .cloned()
            .collect();

        for key in to_remove {
            index.entries.remove(&key);
        }
    }

    fn index_pathspec(&self, path: &Path, index: &mut Index) -> anyhow::Result<()> {
        let rel_path = path.strip_prefix(&self.work_dir)?.to_string_lossy();

        let was_indexed = index.entries.remove(rel_path.as_ref()).is_some();

        match path.metadata() {
            Ok(md) => {
                let obj_hash = Object::write_blob_from_file(self, path)?;
                index.insert_entry(&md, obj_hash, rel_path.into_owned());
            }
            Err(_) => {
                if !was_indexed {
                    bail!("path not found: {}", path.display()); //if wasnt found + wasnt indexed error
                }
            }
        }

        Ok(())
    }

    pub fn remove_paths_from_index(&self, paths: &[PathBuf]) -> anyhow::Result<()> {
        let mut index = self.get_index()?;
        self.remove_pathspecs(paths, &mut index)?;
        self.save_index(index)?;
        Ok(())
    }

    fn remove_pathspecs(&self, paths: &[PathBuf], index: &mut Index) -> anyhow::Result<()> {
        for path in paths {
            if Self::ignore(path) {
                continue;
            }

            if path.is_dir() {
                let children = get_children_of_dir(path)?;
                self.remove_pathspecs(&children, index)?;
            } else if path.is_file() || !self.has_tracked_children(path, index) {
                let rel_path = path.strip_prefix(&self.work_dir)?.to_string_lossy();
                index.entries.remove(rel_path.as_ref());
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_branch(&self) -> anyhow::Result<Option<String>> {
        let head_str = fs::read_to_string(self.data_dir.join("HEAD"))?;

        if let Some(branch) = head_str.trim().strip_prefix("ref: ") {
            let true_branch = branch
                .strip_prefix("refs/heads/")
                .context("head is corrupted")?;

            Ok(Some(true_branch.to_owned()))
        } else {
            Ok(None)
        }
    }

    pub fn get_head_commit(&self) -> anyhow::Result<Commit> {
        let head_ref = self.get_head()?;
        let commit_hash = self
            .resolve_ref(Path::new(&head_ref), 10)
            .context("failed to resolve commit from branch")?;
        let obj = Object::read(self, &commit_hash, true)?;
        Commit::from_object(&obj)
    }
}
