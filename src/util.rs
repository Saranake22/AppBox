use std::fs;
use fs::{File, OpenOptions};
use std::io::{Write, BufRead, BufReader};
use std::path::Path;
use std::io::Result;

pub(crate) fn data_dir() -> Result<std::path::PathBuf>
{
    Ok(Path::new(&std::env::var("HOME").unwrap()).join(".local/share/appbox"))
}

pub(crate) fn fs_write(path: &str, data: &str, append: bool, create: bool) -> std::io::Result<()>
{
    if let Ok(mut file) = OpenOptions::new()
        .append(append)
        .create(create)
        .open(path)
    {
        let _ = writeln!(file, "{}", data);
    }

    Ok(())
}

pub(crate) fn fs_read(path: &str) -> Result<Vec<String>>
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

pub(crate) fn fs_listdir(path: &str) -> Result<Vec<String>>
{
    let files: Vec<String>;
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(format!("for i in {}/*; do echo $i; done", path))
        .output().expect("Failed to read db files");

    println!("{}", String::from_utf8_lossy(&output.stdout).to_string());
    let outputstring = String::from_utf8_lossy(&output.stdout).to_string();
    files = outputstring.lines().map(|line| line.to_string()).collect();

    Ok(files)
}
