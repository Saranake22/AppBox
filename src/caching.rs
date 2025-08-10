use crate::AppInfo;
use std::fs::create_dir_all;
use reqwest::Client;
use std::time::Duration;
use std::collections::BTreeMap;

use crate::util;

pub(crate) fn init()
{
    let path = util::data_dir().unwrap().join("db");
    create_dir_all(path).unwrap();
}

fn gh_client() -> Result<Client, reqwest::Error>
{
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("appbox/1.0")
        .build()?;

    Ok(client)
}

async fn get_webpage_text(url: &str) -> Result<String, reqwest::Error>
{
    let client = gh_client()?;
    let response = client.get(url).send().await?.error_for_status()?;
    let text = response.text().await?;

    Ok(text)
}

pub(crate) async fn read_db_apps() -> std::io::Result<BTreeMap<String, Vec<AppInfo>>>
{
    let path = util::data_dir().unwrap().join("db");
    let dbfiles = util::fs_listdir(&path).await?;
    let mut db: BTreeMap<String, Vec<AppInfo>> = BTreeMap::new();
    for file in dbfiles {
        println!("{}", file.to_str().unwrap());
        let lines = util::fs_readlines(&file).unwrap();

        let appsinfo: Vec<AppInfo> = lines.iter().filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ':').map(str::trim).collect();
            if parts.len() == 2 {
                Some(AppInfo {
                    name: parts[0].to_string(),
                    description: parts[1].trim().to_string(),
                    database: file.file_name().unwrap().to_string_lossy().to_string(),
                    installed: true,
                })
            } else {
                println!("skipping: {:#?}", parts);
                None
            }
        }).collect();

        db.insert(file.file_name().unwrap().to_str().unwrap().to_string(), appsinfo);
    }

    Ok(db)
}

/// Downloads database files `{arch}-appimages` and `{arch}-portable`
pub(crate) async fn create_db() -> std::io::Result<()>
{
    let arch: &str = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "x86") {
        "i686"
    } else {
        ""
    };
    let appimages = match get_webpage_text(&format!("https://raw.githubusercontent.com/ivan-hc/AM/refs/heads/main/programs/{arch}-appimages")).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            String::new()
        },
    };
    let portable = match get_webpage_text(&format!("https://raw.githubusercontent.com/ivan-hc/AM/refs/heads/main/programs/{arch}-portable")).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            String::new()
        },
    };

    let databases = vec![("AppImages", appimages), ("Portable Apps", portable)];

    for (name, db) in databases {
        let applist: Vec<AppInfo> = db.lines()
        .filter_map(|line| {
            let line = line.trim_start_matches('◆').trim();
            let parts: Vec<&str> = line.splitn(2, ':').map(str::trim).collect();
            //println!("parts.len: {}", parts.len());
            if parts.len() == 2 {
                Some(AppInfo {
                    name: parts[0].to_string(),
                     description: parts[1].trim().to_string(),
                     database: name.to_string(),
                     installed: false,
                })
            } else {
                println!("skipping: {:#?}", parts);
                None
            }
        })
        .collect();

        let path = util::data_dir().unwrap().join("db").join(name);
        println!("{}", applist.len());
        for app in applist {
            let data = format!("{}:{}\n", app.name, app.description);
            util::fs_write(
                &path,
                &data, true, true
            ).unwrap();
        }
    }

    Ok(())
}

/// Purges database files in `~/.local/share/appbox/db`
pub(crate) async fn purge_db() -> std::io::Result<()>
{
    let path = util::data_dir()?.join("db");
    for entry in util::fs_listdir(&path).await? {
        util::fs_write(&entry.as_path(), "", false, true)?;
    }

    Ok(())
}
