use anyhow::{Context, anyhow, bail};
use flate2::write::ZlibEncoder;
use flate2::{Compression, read::ZlibDecoder};
use hex::encode;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io::Read};

use crate::structures::repo::Repo;

pub struct Object {
    pub data: Vec<u8>,
    pub object_type: ObjectType,
    pub size: usize,
}

type ParsedObject = (HashMap<String, Vec<String>>, Option<String>);

impl Object {
    pub fn new(data: Vec<u8>, object_type: ObjectType) -> Self {
        let size = data.len();
        Self {
            data,
            object_type,
            size,
        }
    }

    fn decompress(content: &[u8]) -> anyhow::Result<Vec<u8>> {
        let mut data = Vec::new();
        ZlibDecoder::new(content).read_to_end(&mut data)?;
        Ok(data)
    }

    fn compress(header: &[u8], content: &[u8]) -> anyhow::Result<Vec<u8>> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(header)?;
        encoder.write_all(content)?;
        let result = encoder.finish()?;
        Ok(result)
    }

    fn parse_header(data: &[u8]) -> anyhow::Result<(ObjectType, usize, &[u8])> {
        let null = data
            .iter()
            .position(|&c| c == 0)
            .context("missing null byte in header")?;

        let header = str::from_utf8(&data[..null]).context("header is not valid UTF-8")?;
        let (ty, size_str) = header
            .split_once(' ')
            .context("invalid object header format")?;

        let size: usize = size_str.parse().context("invalid object size")?;
        let body = &data[null + 1..];

        Ok((ObjectType::from_str(ty)?, size, body))
    }

    fn hash_to_path(repo: &Repo, hash: &str) -> PathBuf {
        repo.data_dir
            .join("objects")
            .join(&hash[..2])
            .join(&hash[2..])
    }


    fn find_object(repo: &Repo, name: &str) -> anyhow::Result<PathBuf> {
        if name == "HEAD"{
            return Ok(repo.data_dir.join("HEAD"));
        };

        if name.len() == 40 {
            let path = Self::hash_to_path(repo, name);
            if path.is_file() {
                return Ok(path);
            }
        };

        if name.len() > 4{
            let objects_dir = repo.data_dir.join("objects").join(&name[..2]);
            if objects_dir.is_dir() {
                let paths = fs::read_dir(objects_dir)?.filter_map(|entry|{
                    let entry = entry.ok()?;
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_str()?;
                    if file_name_str.starts_with(&name[2..]) {
                        return Some(entry.path());
                    }
                    None
                }).collect::<Vec<PathBuf>>();

                if paths.len() == 1 {
                    return Ok(paths[0].clone());
                } else if paths.len() > 1 {
                    anyhow::bail!("ambiguous object name: {}", name);
                }
            };

        };

        let tag = repo.get_tag_path(name);

        if let Ok(tag_path) = tag {
            let result = repo.resolve_ref(&repo.data_dir.join("tags").join(tag_path), 10).context("failed to resolve tag reference")?;
            let result_path = Self::hash_to_path(repo, &result);
            return Ok(PathBuf::from(result_path));
        };

        bail!("object not found: {}", name);

    }
    

    pub fn read(repo: &Repo, hash: &str) -> anyhow::Result<Object> {
        let path = Self::find_object(repo, hash)?;

        let content = fs::read(path).context("failed to read object file")?;

        let data = Self::decompress(&content)?;

        let (ty, size, body) = Self::parse_header(&data)?;

        if body.len() != size {
            anyhow::bail!("object size mismatch");
        }

        let obj = Self::new(body.to_vec(), ty);

        Ok(obj)
    }

    fn header(&self) -> Vec<u8> {
        format!("{} {}\0", self.object_type, self.size).into_bytes()
    }

    pub fn hash(&self) -> [u8; 20] {
        let mut hasher = Sha1::new();
        hasher.update(self.header());
        hasher.update(&self.data);
        let hash = hasher.finalize();
        hash.into()
    }

    fn make_readonly(path: &Path) -> anyhow::Result<()> {
        let metadata = fs::metadata(path)?;
        let mut permissions = metadata.permissions();

        permissions.set_readonly(true);

        fs::set_permissions(path, permissions)?;

        Ok(())
    }

    pub fn write(&self, repo: &Repo) -> anyhow::Result<[u8; 20]> {
        let self_hash = self.hash();

        let hash = encode(self_hash);
        let path = repo
            .data_dir
            .join("objects")
            .join(&hash[..2])
            .join(&hash[2..]);

        // the file is immutable so if we created it before just dont create it
        if path.is_file() {
            return Ok(self_hash);
        }

        let parent = path.parent().context("failed to get parent")?;
        fs::create_dir_all(parent)?;

        let header = self.header();
        let compressed = Self::compress(&header, &self.data)?;
        fs::write(&path, compressed)?;

        //make the file immutable
        Self::make_readonly(&path)?;

        Ok(self_hash)
    }

    pub fn write_blob_from_file(repo: &Repo, path: &Path) -> anyhow::Result<[u8; 20]> {
        let content = fs::read(path).context("failed to read object file")?;
        let object = Object::new(content, ObjectType::Blob);
        object.write(repo)
    }

    /// parses a git object into key value
    /// returns the hashmap of the keyvalue + the rest after \n\n
    pub fn parse_key_value(data: &[u8]) -> anyhow::Result<ParsedObject> {
        let content = std::str::from_utf8(data)?;
        let (headers_str, rest_str) = content.split_once("\n\n").unwrap_or((content, ""));

        let mut headers: HashMap<String, Vec<String>> = HashMap::new();

        for line in headers_str.lines() {
            if let Some((key, value)) = line.split_once(' ') {
                headers
                    .entry(key.to_string())
                    .or_default()
                    .push(value.to_string());
            }
        }

        let rest_str = (!rest_str.is_empty()).then(|| rest_str.trim().to_string());

        Ok((headers, rest_str))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectType {
    Blob,
    Commit,
    Tree,
    Tag,
}

impl ObjectType {
    pub fn from_str(str: &str) -> anyhow::Result<ObjectType> {
        match str {
            "blob" => Ok(ObjectType::Blob),
            "commit" => Ok(ObjectType::Commit),
            "tree" => Ok(ObjectType::Tree),
            "tag" => Ok(ObjectType::Tag),
            _ => Err(anyhow!("invalid object type: {}", str)),
        }
    }
}

impl std::fmt::Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Blob => write!(f, "blob"),
            ObjectType::Commit => write!(f, "commit"),
            ObjectType::Tree => write!(f, "tree"),
            ObjectType::Tag => write!(f, "tag"),
        }
    }
}
