mod repository;
mod testutil;

use std::path::Path;
use std::{fs::File, io::Write};

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use ini::Ini;

use repository::Repository;

fn main() -> Result<()> {
    let args = Cli::parse();

    let Some(cmd) = args.command else {
        return Err(anyhow!("No command provided"));
    };

    match cmd {
        Commands::Init { path } => {
            let path = Path::new(&path);
            Command::create(path)?;
        }
    };

    Ok(())
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg()]
        path: String,
    },
}

struct Command {}

impl Command {
    pub fn create(path: &Path) -> Result<Repository> {
        let root = path;
        let repo = Repository::new(path);
        if root.is_file() {
            return Err(anyhow!("Not a directory: {:?}", root));
        }
        repo.mkdir(&root)?;

        repo.mkdir("branches")?;
        repo.mkdir("objects")?;
        repo.mkdir("refs/heads")?;
        repo.mkdir("refs/tags")?;

        let description = repo
            .file("description")
            .ok_or(anyhow!("can't use description file"))?;
        match File::create(description) {
            Err(e) => return Err(e.into()),
            Ok(mut file) => file.write(
                b"Unnamed repository; edit this file 'description' to name the repository.\n",
            )?,
        };

        let head = repo.file("HEAD").ok_or(anyhow!("can't use HEAD file"))?;
        match File::create(head) {
            Err(e) => return Err(e.into()),
            Ok(mut file) => file.write(b"ref: refs/heads/main\n")?,
        };

        let config = repo
            .file("config")
            .ok_or(anyhow!("can't use config file"))?;
        match File::create(&config) {
            Err(e) => return Err(e.into()),
            Ok(_) => {
                let cfg = default_config();
                cfg.write_to_file(&config)?;
            }
        }

        return Ok(repo);
    }
}

fn default_config() -> Ini {
    let mut cfg = Ini::new();
    cfg.with_section(Some("core"))
        .set("repositoryformatversion", "0")
        .set("filemode", "false")
        .set("bare", "false");
    return cfg;
}

#[cfg(test)]
mod test {
    use std::path;

    use testutil::with_tempdir;

    use super::*;

    #[test]
    fn command_create() {
        with_tempdir(|p: &path::Path| {
            Command::create(p)?;

            assert!(p.join(".git/branches").is_dir());
            assert!(p.join(".git/objects").is_dir());
            assert!(p.join(".git/refs/heads").is_dir());
            assert!(p.join(".git/refs/tags").is_dir());

            assert!(p.join(".git/description").is_file());
            assert!(p.join(".git/HEAD").is_file());
            assert!(p.join(".git/config").is_file());

            Ok(())
        });
    }
}
