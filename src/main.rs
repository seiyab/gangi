use std::path::{self, Path};
use std::{fs::File, io::Write};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use ini::Ini;

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

struct Repository {
    worktree: Box<Path>,
    gitdir: Box<Path>,
}

impl Repository {
    pub fn new(path: &Path) -> Self {
        let worktree = path;
        let gitdir = worktree.join(".git");
        return Self {
            worktree: worktree.into(),
            gitdir: gitdir.into_boxed_path(),
        };
    }

    fn file<P: AsRef<Path>>(&self, relative_path: P) -> Option<Box<Path>> {
        let path = self.path(relative_path);
        let par = path.parent()?;
        if !par.exists() {
            return None;
        };
        return Some(path);
    }

    fn dir<P: AsRef<Path>>(&self, relative_path: P) -> Option<Box<Path>> {
        let path = self.path(relative_path);
        if !path.exists() {
            return None;
        }
        if !path.is_dir() {
            return None;
        }
        return Some(path);
    }

    fn path<P: AsRef<Path>>(&self, path: P) -> Box<Path> {
        return self.gitdir.join(path).into_boxed_path();
    }

    fn mkdir<P: AsRef<Path> + std::fmt::Debug + Clone>(&self, path: P) -> Result<()> {
        let path_buf = self
            .path(path)
            .to_path_buf()
            .components()
            .fold(path::PathBuf::new(), |acc, c| acc.join(c));
        std::fs::create_dir_all(&path_buf)
            .with_context(|| format!("failed to make directory {:?}", path_buf))?;
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use std::env;
    use std::panic;
    use std::path;

    use anyhow::Result;
    use rand;
    use rand::distributions::DistString;

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

    #[test]
    fn mkdir() {
        with_tempdir(|p: &path::Path| {
            let r = Repository::new(p);
            assert_eq!(r.mkdir(".").err().map(|e| e.to_string()), None);
            assert!(p.join(".git").is_dir());
            Ok(())
        });
    }

    #[test]
    fn file() {
        with_tempdir(|p: &path::Path| {
            let r = Repository::new(p);

            r.mkdir(".")?;

            assert_eq!(
                r.file("description")
                    .and_then(|d| d.to_str().map(|s| s.to_string())),
                p.to_str().map(|p| format!("{}/.git/description", p)),
            );

            Ok(())
        })
    }

    fn with_tempdir<F>(f: F)
    where
        F: FnOnce(&path::Path) -> Result<()> + std::panic::UnwindSafe,
    {
        let r = rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 8);
        let td = env::temp_dir().join(format!("test-{}", r));
        let p = panic::catch_unwind(|| {
            assert!(std::fs::create_dir(&td).is_ok());
            assert_ok(f(&td));
        });
        let rda = std::fs::remove_dir_all(&td)
            .map_err(anyhow::Error::from)
            .with_context(|| format!("failed to remove {:?}", td));
        assert_ok(rda);
        assert!(p.is_ok());
    }

    fn assert_ok<T>(r: Result<T>) {
        assert_eq!(r.err().map(|e| e.to_string()), None,);
    }
}
