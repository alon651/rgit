use std::{env, io::Write};

use anyhow::anyhow;
use clap::Args;

use crate::structures::{object::Object, repo::Repo};

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct CatFileActions {
    ///Pretty print the file content
    #[arg(short = 'p')]
    pretty: bool,
    ///Output the type of the object(blob,tree,commit)
    #[arg(short = 't')]
    typename: bool,
    ///Output the object size
    #[arg(short = 's')]
    size: bool,
}

impl CatFileActions {
    pub fn to_mode(&self) -> CatFileMode {
        if self.pretty {
            CatFileMode::Pretty
        } else if self.typename {
            CatFileMode::Typename
        } else if self.size {
            CatFileMode::Size
        } else {
            panic!("must have at least one CatFile action, clap was supposed to stop it")
        }
    }
}

pub enum CatFileMode {
    Pretty,
    Typename,
    Size,
}

pub fn exec(hash: &str, mode: CatFileMode) -> anyhow::Result<()> {
    let repo = Repo::find(&env::current_dir()?).ok_or_else(|| anyhow!("didn't find a repo"))?;
    let obj = Object::read(&repo, hash)?;

    match mode {
        CatFileMode::Pretty => std::io::stdout().write_all(&obj.data)?, //write it directly to the stdout
        CatFileMode::Typename => println!("{}", obj.object_type),
        CatFileMode::Size => println!("{}", obj.size),
    };

    Ok(())
}
