mod util;
mod caching;

mod types;
use types::AppEntry;

use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{glib, gio, Application, ApplicationWindow, 
        Label, Box, SearchEntry, Orientation::Vertical, Orientation::Horizontal
};

use std::thread;

//use chrono::Local;

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();
    
    app.connect_activate(callback_activate);

    app.run()
}

fn callback_activate(application: &gtk::Application)
{
    build_ui(application);
    caching::init();
}

fn build_ui(application: &gtk::Application)
{
    let window = ApplicationWindow::builder()
        .application(application)
        .default_width(800)
        .default_height(600)
        .title("hello world!")
        .build();
    
    let winbox = Box::new(Vertical, 4);
    let _menu = gio::Menu::new();
    
    let container = Box::builder()
        .orientation(Vertical)
        .spacing(6)
        .margin_top(12).margin_bottom(12).margin_start(12).margin_end(12)
        .valign(gtk::Align::Start)
        .build();

    let time = do_magic().unwrap();//current_time();
    let tlabel = Label::default();
    tlabel.set_text(&time);

    let search_container = Box::builder()
        .orientation(Horizontal)
        .spacing(2)
        .build();

    let search_entry = SearchEntry::builder()
        .placeholder_text("Search for more apps to install")
        .build();
    search_entry.set_hexpand(true);
    
    //container.append(&search_entry);
    search_container.append(&search_entry);

    let test_button = gtk::Button::with_label("test");
    test_button.connect_clicked(move |_| {
        //println!("{}", do_magic());
        let _ = parse_magic();
    });
    //container.append(&test_button);
    let update_button = gtk::Button::with_label("Update Lists");
    update_button.connect_clicked(move |_| {
        let _ = thread::spawn(move || {
            /**/
            let _ = caching::create_db();
            println!("===============================");
            let _ = caching::get_apps();
        });
    });
    search_container.append(&test_button);
    search_container.append(&update_button);
    container.append(&search_container);
    container.append(&tlabel);

    winbox.append(&container);

    window.set_child(Some(&winbox));
    window.present();

    /*let tick = move || {
        let time = current_time();
        tlabel.set_text(&time);
        glib::ControlFlow::Continue
    };

    glib::timeout_add_seconds_local(1, tick);*/
}
/*
fn current_time() -> String
{
    format!("{}", Local::now().format("%Y-%m-%d %H:%M:%S"))
}
*/

fn get_app_num() -> u32
{
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg("am files --less")
        .output().expect("failed");

    let numstr = String::from_utf8_lossy(&output.stdout).to_string();
    let num = numstr.trim().parse().unwrap();

    num
}

fn do_magic() -> std::io::Result<String>
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

    Ok(mystring)
}

fn parse_magic()
{
    let bruh = do_magic().unwrap();
    let apps = parse_apps(&bruh);
    for app in apps
    {
        println!("=================================================");
        println!("App: {}", &app.name);
        println!("Version: {}", &app.version);
        println!("Type: {}", &app.kind);
        println!("Size: {}", &app.size);
    }
}

fn parse_apps(output: &str) -> Vec<AppEntry> {
    output.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.trim().split('|').map(str::trim).collect();
            println!("parts.len: {}", parts.len());
            if parts.len() == 4 {
                Some(AppEntry {
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
