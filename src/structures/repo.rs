use std::{
    fs,
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

    pub fn resolve_ref(&self, path: &str, depth: usize) -> Option<String> {
        if depth == 0 {
            return None;
        }

        let ref_path = self.data_dir.join(path);
        let content = fs::read_to_string(ref_path).ok()?;

        match content.trim().strip_prefix("ref: ") {
            Some(content) => self.resolve_ref(content, depth - 1),
            None => Some(content),
        }
    }
}
