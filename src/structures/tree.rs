use std::{fmt::Display, io::Write};

use anyhow::{Context, anyhow, bail, ensure};
use hex::encode;

use crate::structures::{
    object::{Object, ObjectType},
    repo::Repo,
};

#[derive(Debug)]
pub struct TreeEntry {
    pub mode: u32,
    pub name: String,
    pub hash: [u8; 20],
}

#[derive(Debug)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

impl Tree {
    /// read characters from data until reaching the delimiter,
    /// starting at position pos and advancing pos past the delimiter
    fn read_and_advance<'a>(data: &mut &'a [u8], delimiter: u8) -> anyhow::Result<&'a [u8]> {
        let pos = data
            .iter()
            .position(|&b| b == delimiter)
            .ok_or_else(|| anyhow!("Delimiter not found"))?;

        let (found, rest) = data.split_at(pos);
        *data = &rest[1..]; // Skip the delimiter itself
        Ok(found)
    }

    pub fn new(entries: Vec<TreeEntry>) -> Self {
        let mut entries = entries;
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Self { entries }
    }

    pub fn from_object(obj: &Object) -> anyhow::Result<Self> {
        ensure!(
            obj.object_type == ObjectType::Tree,
            "Object must be Tree, got {}",
            obj.object_type
        );

        let mut entries = Vec::new();
        let mut data = &obj.data[..];

        while !data.is_empty() {
            let mode_bytes = Self::read_and_advance(&mut data, b' ')?.to_vec();
            let name_bytes = Self::read_and_advance(&mut data, b'\0')?.to_vec();

            if data.len() < 20 {
                bail!("Invalid tree object: invalid hash");
            }
            let (hash_bytes, rest) = data.split_at(20);
            data = rest;

            let mode_str = std::str::from_utf8(&mode_bytes)?;
            let mode = u32::from_str_radix(mode_str, 8).context("invalid mode")?;

            entries.push(TreeEntry {
                mode,
                name: String::from_utf8(name_bytes)?,
                hash: hash_bytes
                    .to_owned()
                    .try_into()
                    .map_err(|_| anyhow!("invalid hash length"))?,
            });
        }

        Ok(Self::new(entries))
    }

    pub fn write(&self, repo: &Repo) -> anyhow::Result<[u8; 20]> {
        let mut content = Vec::new();

        for entry in &self.entries {
            //apparently you can write into a vector with write! and it will append to it
            write!(&mut content, "{:o} {}", entry.mode, entry.name)?;
            content.push(0);
            content.write_all(&entry.hash)?;
        }

        Object::new(content, ObjectType::Tree).write(repo)
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.entries
            .iter()
            .try_for_each(|entry| writeln!(f, "{}", entry))
    }
}

impl FromIterator<TreeEntry> for Tree {
    fn from_iter<T: IntoIterator<Item = TreeEntry>>(iter: T) -> Self {
        let mut entries = Vec::new();

        for entry in iter {
            entries.push(entry);
        }

        Self::new(entries)
    }
}

impl Display for TreeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //git displays the mode with 0 at the start so we do it
        let mode = if self.mode == 0o40000 {
            "040000".to_string()
        } else {
            format!("{:o}", self.mode)
        };

        //match the mode to his object type
        let ty = match self.mode {
            0o100644 | 0o100755 | 0o120000 => "blob",
            0o40000 => "tree",
            _ => return Err(std::fmt::Error),
        };

        //write it
        write!(f, "{} {} {}    {}", mode, ty, encode(self.hash), self.name)
    }
}
