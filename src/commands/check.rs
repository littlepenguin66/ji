use crate::error::Result;
use crate::store::path;
use std::path::PathBuf;

pub fn run(input: Option<PathBuf>, deep: bool) -> Result<()> {
    let target = match input {
        Some(p) => p,
        None => auto_discover()?,
    };
    crate::archive::verify_archive(&target, deep)
}

fn auto_discover() -> Result<PathBuf> {
    let dir = path::data_dir();
    let mut candidates: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|_| crate::error::Error::Archive(format!("no .ji files found in {}", dir.display())))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "ji"))
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            Some((e.path(), meta.modified().ok()?))
        })
        .collect();
    candidates.sort_by(|a, b| b.1.cmp(&a.1));
    candidates
        .first()
        .map(|(p, _)| p.clone())
        .ok_or_else(|| crate::error::Error::Archive(format!("no .ji files found in {}", dir.display())))
}
