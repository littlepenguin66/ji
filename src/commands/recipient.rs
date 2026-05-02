use crate::error::Result;
use std::path::PathBuf;

pub fn run_list(input: PathBuf) -> Result<()> {
    let recipients = crate::archive::list_archive_recipients(&input)?;

    if recipients.is_empty() {
        println!("(no recipients found)");
    } else {
        for r in &recipients {
            println!("{r}");
        }
    }

    Ok(())
}

pub fn run_add(key: String, input: PathBuf) -> Result<()> {
    crate::archive::add_archive_recipient(&input, &key)
}

pub fn run_remove(key: String, input: PathBuf) -> Result<()> {
    crate::archive::remove_archive_recipient(&input, &key)
}
