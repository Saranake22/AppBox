use gtk4 as gtk;
use gtk::glib;

#[derive(Debug, Clone, glib::Boxed)]
#[boxed_type(name = "AppEntry")]
pub(crate) struct AppEntry
{
    pub name: String,
    pub version: String,
    pub kind: String,
    pub size: String,
}

pub(crate) struct AppInfo
{
    pub name: String,
    pub description: String,
}
