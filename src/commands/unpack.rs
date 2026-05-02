use crate::archive;
use crate::error::Result;
use std::path::PathBuf;

pub fn run(
    input: PathBuf,
    dry_run: bool,
    force: bool,
    interactive: bool,
    backup: bool,
) -> Result<()> {
    if !input.exists() {
        return Err(crate::error::Error::Archive(format!(
            "file not found: {}",
            input.display()
        )));
    }

    let restored = archive::unpack_archive(&input, dry_run, force, interactive, backup)?;

    if restored == 0 {
        println!("(nothing to restore)");
    } else if dry_run {
        println!("Would restore {restored} file(s)");
    } else {
        println!("Restored {restored} file(s)");
    }

    Ok(())
}
