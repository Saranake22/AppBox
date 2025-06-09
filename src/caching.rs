use std::process::Command;
use crate::types::AppInfo;
use std::fs::create_dir_all;

use crate::util;
use crate::types;

pub(crate) fn init()
{
    let path = util::data_dir().unwrap();
    create_dir_all(path.join("db")).unwrap();
}

pub(crate) fn get_apps() -> std::io::Result<()>
{
    let path = util::data_dir().unwrap().join("db");
    println!("{}", path.to_string_lossy().into_owned());
    let dbfiles = util::fs_listdir(path.to_string_lossy().into_owned().as_str()).unwrap();

    println!("{:?}", dbfiles);

    for file in dbfiles
    {
        let flines = util::fs_read(&file).unwrap();
        for line in flines
        {
            println!("{}", line);
        }
    }

    Ok(())
}

pub(crate) fn create_applist() -> std::io::Result<Vec<AppInfo>>
{
    let output = Command::new("bash")
        .arg("-c")
        .arg("am list --all | cat")
        .output().expect("Failed to fetch app list from am.");

    let bruh = String::from_utf8_lossy(&output.stdout).to_string();
    let outputstr = bruh.as_str();

    let applist = outputstr.lines()
    .filter_map(|line| {
        let parts: Vec<&str> = line.trim().split(':').map(str::trim).collect();
        println!("parts.len: {}", parts.len());
        if parts.len() == 2 {
            Some(AppInfo {
                name: parts[0].trim_start_matches('◆').trim().to_string(),
                description: parts[1].trim().to_string(),
            })
        } else {
            None
        }
    })
    .collect();

    Ok(applist)
}

pub(crate) fn create_db() -> std::io::Result<()>
{
    let bruh = create_applist().unwrap();
    let path = util::data_dir().unwrap().join("db");
    for app in bruh
    {
        let data = format!("{}:{}", app.name, app.description);
        let path = path.join(app.name.chars().next().unwrap().to_string());
        println!("{}", data.as_str());
        println!("{}", path.to_string_lossy().into_owned());
        util::fs_write(
            &path.to_string_lossy().into_owned(),
            &data, true, true
        ).unwrap();
    }

    Ok(())
}

pub(crate) fn purge_db() -> std::io::Result<()>
{
    let _ = std::process::Command::new("bash")
        .arg("-c")
        .arg("rm -rf ~/.local/share/appbox/db/*")
        .output().expect("Failed to purge db");

    Ok(())
}
