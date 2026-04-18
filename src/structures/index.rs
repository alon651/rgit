use anyhow::{anyhow, bail, ensure};

#[derive(Debug)]
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

#[derive(Debug)]
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

pub struct Index {
    pub entries: Vec<IndexEntry>,
    pub version: u32,
}

impl Index {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            version: 2,
        }
    }

    pub fn read(path: &std::path::Path) -> anyhow::Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut buf)?;

        let mut index = Self::new();

        anyhow::ensure!(&buf[0..4] == b"DIRC", "invalid index file");

        let version = u32::from_be_bytes(buf[4..8].try_into()?);
        anyhow::ensure!(version == 2, "unsupported index version");

        let count = u32::from_be_bytes(buf[8..12].try_into()?);

        let mut entries: Vec<IndexEntry> = Vec::with_capacity(count as usize);

        let mut idx = 12;
        for _ in 0..count {
            let entry_start = idx;

            let ctime = u32::from_be_bytes(buf[idx..idx + 4].try_into()?);
            let _ctime_nsec = u32::from_be_bytes(buf[idx + 4..idx + 8].try_into()?);
            let mtime = u32::from_be_bytes(buf[idx + 8..idx + 12].try_into()?);
            let _mtime_nsec = u32::from_be_bytes(buf[idx + 12..idx + 16].try_into()?);
            let dev = u32::from_be_bytes(buf[idx + 16..idx + 20].try_into()?);
            let ino = u32::from_be_bytes(buf[idx + 20..idx + 24].try_into()?);
            let unused = u16::from_be_bytes(buf[idx + 24..idx + 26].try_into()?);
            ensure!(unused == 0, "invalid useless file {}", unused);
            let mode = u16::from_be_bytes(buf[idx + 26..idx + 28].try_into()?); // get 2 bytes(16 bits)

            let perms = mode & 0b0000000111111111; // get the bottom nine bits

            let mode_type = ModeType::from_u16(mode)?; // get the upper 4 bits, the middle 3 bits are ignroed

            let uid = u32::from_be_bytes(buf[idx + 28..idx + 32].try_into()?);
            let gid = u32::from_be_bytes(buf[idx + 32..idx + 36].try_into()?);
            let fsize = u32::from_be_bytes(buf[idx + 36..idx + 40].try_into()?);
            let sha1_bytes: [u8; 20] = buf[idx + 40..idx + 60].try_into()?;

            let flags = u16::from_be_bytes(buf[idx + 60..idx + 62].try_into()?);
            let assume_valid = flags & 0b1000000000000000 != 0; // the first bit here is the assume valid bit - we take it here

            let extended_flag = flags >> 14 & 1;
            ensure!(
                extended_flag == 0,
                "extended flag invalid must be 0 but {}",
                extended_flag
            );

            let stage = flags >> 12; // the third and fourth bits are the multilevel stage bits
            let name_len = flags & 0x0FFF; //the remaining 12 bits are the name_len

            idx += 62;
            let name = String::from_utf8(buf[idx..idx + (name_len as usize)].to_vec())?;

            ensure!(
                buf[idx + (name_len as usize)] == 0,
                "invalid index entry name"
            );

            idx += name_len as usize + 1;
            while (idx - entry_start) % 8 != 0 {
                idx += 1;
            }

            let entry = IndexEntry {
                ctime,
                mtime,
                dev,
                ino,
                mode: mode as u32,
                tp: mode_type,
                perms: perms as u32,
                uid,
                gid,
                fsize,
                sha1: sha1_bytes,
                assume_valid,
                stage,
                name,
            };
            entries.push(entry);
        }

        index.entries = entries;
        index.version = version;
        Ok(index)
    }
}
