use std::{
    fs::{self, File, Metadata},
    os::unix::fs::MetadataExt,
    path::Path,
};

use anyhow::{Context, bail};
use rkyv::{Archive, Deserialize, Serialize, access, deserialize, rancor::Error, to_bytes};
use std::collections::BTreeMap;
#[derive(Archive, Serialize, Deserialize, Debug)]
pub enum ModeType {
    RegularFile,
    GitLink,
    SymbolicLink,
}

impl ModeType {
    pub fn from_u16(val: u16) -> anyhow::Result<Self> {
        let mode_bits = val >> 12;
        match mode_bits {
            0b1000 => Ok(ModeType::RegularFile),
            0b1010 => Ok(ModeType::SymbolicLink),
            0b1110 => Ok(ModeType::GitLink),
            res => bail!("invalid mode {:b}", res),
        }
    }
}

#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct IndexEntry {
    pub ctime: u32,
    pub mtime: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub tp: ModeType,
    pub perms: u32,
    pub uid: u32,
    pub gid: u32,
    pub fsize: u32,
    pub sha1: [u8; 20],
    pub assume_valid: bool,
    pub stage: u16,
    pub name: String,
}

#[derive(Archive, Serialize, Deserialize, Debug)]
pub struct Index {
    pub entries: BTreeMap<String, IndexEntry>,
    pub version: u32,
}

impl Index {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            version: 2,
        }
    }

    pub fn read(path: &std::path::Path) -> anyhow::Result<Self> {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut buf)?;

        let archived = access::<ArchivedIndex, Error>(&buf).context("parsing the index file")?;
        let deserialized =
            deserialize::<Index, Error>(archived).context("deserializing the index file")?;
        Ok(deserialized)
    }

    pub fn save_index(&self, path: &Path) -> anyhow::Result<()> {
        let bytes = to_bytes::<Error>(self)?;
        fs::write(path, bytes)?;
        Ok(())
    }

    pub fn insert_entry(&mut self, md: &Metadata, sha: [u8; 20], rel_path: String) {
        let entry = IndexEntry::from_metadata(md, sha, rel_path.clone());
        self.entries.insert(rel_path, entry);
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexEntry {
    fn from_metadata(md: &Metadata, sha: [u8; 20], rel_path: String) -> Self {
        Self {
            ctime: md.ctime() as u32,
            mtime: md.mtime() as u32,
            dev: md.dev() as u32,
            ino: md.ino() as u32,
            mode: md.mode(),
            tp: ModeType::from_u16((md.mode() >> 12) as u16).unwrap_or(ModeType::RegularFile),
            perms: md.mode() & 0o777,
            uid: md.uid(),
            gid: md.gid(),
            fsize: md.size() as u32,
            sha1: sha,
            assume_valid: false,
            stage: 0,
            name: rel_path,
        }
    }
}
