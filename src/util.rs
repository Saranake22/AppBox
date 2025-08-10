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

fn get_app_num() -> u32
{
    let output = std::process::Command::new("bash")
    .arg("-c")
    .arg("am files --less")
    .output().expect("failed");

    let numstr = String::from_utf8_lossy(&output.stdout).to_string();
    numstr.trim().parse().unwrap()
}

pub(crate) fn get_installed_apps() -> Vec<crate::AppEntry>
{
    let output = std::process::Command::new("bash")
    .arg("-c")
    .arg("am files")
    .output().expect("failed");

    let bruh = String::from_utf8_lossy(&output.stdout).to_string();
    let outputstr = bruh.trim();

    let v: Vec<&str> = outputstr.split('\n').collect();
    let mut mystring = String::new();

    let num = get_app_num();
    for i in 4..4+num {
        mystring.push_str(v[i as usize].trim());
        if i != 3+num {
            mystring.push_str("\n");
        }
    }

    mystring.lines()
    .filter_map(|line| {
        let parts: Vec<&str> = line.trim().split('|').map(str::trim).collect();
        println!("parts.len: {}", parts.len());
        if parts.len() == 4 {
            Some(crate::AppEntry {
                name: parts[0].trim_start_matches('◆').trim().to_string(),
                 version: parts[1].to_string(),
                 kind: parts[2].to_string(),
                 size: parts[3].to_string(),
            })
        } else {
            None
        }
    })
    .collect()
}
