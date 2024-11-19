use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub struct Repository {
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

    pub fn file<P: AsRef<Path>>(&self, relative_path: P) -> Option<Box<Path>> {
        let path = self.path(relative_path);
        let par = path.parent()?;
        if !par.exists() {
            return None;
        };
        return Some(path);
    }

    pub fn dir<P: AsRef<Path>>(&self, relative_path: P) -> Option<Box<Path>> {
        let path = self.path(relative_path);
        if !path.exists() {
            return None;
        }
        if !path.is_dir() {
            return None;
        }
        return Some(path);
    }

    pub fn path<P: AsRef<Path>>(&self, path: P) -> Box<Path> {
        return self.gitdir.join(path).into_boxed_path();
    }

    pub fn mkdir<P: AsRef<Path> + std::fmt::Debug + Clone>(&self, path: P) -> Result<()> {
        let path_buf = self
            .path(path)
            .to_path_buf()
            .components()
            .fold(PathBuf::new(), |acc, c| acc.join(c));
        std::fs::create_dir_all(&path_buf)
            .with_context(|| format!("failed to make directory {:?}", path_buf))?;
        return Ok(());
    }
}
#[cfg(test)]
mod test {
    use crate::testutil::with_tempdir;
    use std::path;

    use super::*;

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
}
