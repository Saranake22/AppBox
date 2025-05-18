use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Label};

use chrono::Local;

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
}

fn build_ui(application: &gtk::Application)
{
    let window = ApplicationWindow::builder()
        .application(application)
        .default_width(300)
        .default_height(200)
        .title("hello world!")
        .build();

    let time = do_magic();//current_time();
    let tlabel = Label::default();
    tlabel.set_text(&time);

    window.set_child(Some(&tlabel));
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
// https://github.com/ivan-hc/AM/issues/1215#issuecomment-2556811547

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

fn do_magic() -> String
{
    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg("am files")
        .output().expect("failed");
    
    let mut bruh = String::from_utf8_lossy(&output.stdout).to_string();
    bruh.push_str("                                                                                         ");
    let outputstr = bruh.trim();

    let v: Vec<&str> = outputstr.split('\n').collect();
    let mut mystring = String::new();

    let num = get_app_num();
    for i in 4..4+num {
        mystring.push_str(v[i as usize]);
        mystring.push_str("\n");
    }

    mystring
}