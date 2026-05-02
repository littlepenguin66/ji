use crate::error::Result;
use crate::store::path;
use std::path::PathBuf;

pub fn run(input: Option<PathBuf>, deep: bool) -> Result<()> {
    let target = match input {
        Some(p) => p,
        None => path::discover_ji()?,
    };
    crate::archive::verify_archive(&target, deep)
}
