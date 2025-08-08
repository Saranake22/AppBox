use std::fs;
use fs::{File, OpenOptions};
use std::io::{Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::io::Result;

pub(crate) fn data_dir() -> Result<PathBuf>
{
    Ok(Path::new(&std::env::var("HOME").unwrap()).join(".local/share/appbox"))
}

pub(crate) fn fs_write(path: &Path, data: &str, append: bool, create: bool) -> std::io::Result<()>
{
    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(create)
        .truncate(!append)
        .append(append)
        .open(path)
    {
        let _ = write!(file, "{}", data);
    }

    Ok(())
}

pub(crate) fn fs_readlines(path: &Path) -> Result<Vec<String>>
{
    let path = Path::new(path);
    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line?);
    }

    Ok(lines)
}

pub(crate) async fn fs_listdir(path: &Path) -> Result<Vec<PathBuf>>
{
    let mut files: Vec<PathBuf> = Vec::new();
    let mut read = tokio::fs::read_dir(path).await?;

    while let Some(entry) = read.next_entry().await? {
        files.push(entry.path());
    }

    Ok(files)
}
