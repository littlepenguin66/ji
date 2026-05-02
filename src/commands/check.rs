use crate::error::Result;
use std::path::PathBuf;

pub fn run(input: PathBuf, deep: bool) -> Result<()> {
    crate::archive::verify_archive(&input, deep)
}
