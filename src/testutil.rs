use rand;
use rand::distributions::DistString;
use std::env;
use std::panic;
use std::path::Path;

use anyhow::{Context, Result};

pub fn with_tempdir<F>(f: F)
where
    F: FnOnce(&Path) -> Result<()> + std::panic::UnwindSafe,
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
