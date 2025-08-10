use std::collections::BTreeMap;

use futures_util::FutureExt;
use relm4::gtk::prelude::{EditableExt, ListItemExt};

use relm4::{
    gtk, component::{AsyncComponent, AsyncComponentParts, AsyncComponentSender},
    typed_view::list::{RelmListItem, TypedListView},
    RelmApp, RelmWidgetExt
};

use gtk::{Orientation, Align};
use gtk::prelude::{BoxExt, GridExt, WidgetExt, GtkWindowExt, OrientableExt, ButtonExt, CheckButtonExt, EntryExt};

mod caching;
mod util;


fn main() -> std::io::Result<()>
{
    /*if let Err(e) = reqwest::Client::builder().build() {
        eprintln!("Client build failed: {:?}", e);
        let mut src = e.source();
        while let Some(s) = src {
            eprintln!("Caused by: {:?}", s);
            src = s.source();
        }
        return Err(e);
    }*/
    caching::init();
    RelmApp::new("wolfpack.kazam.appbox").run_async::<App>(());

    Ok(())
}

#[derive(Debug, Clone)]
pub struct AppInfo
{
    pub name: String,
    pub description: String,
    pub database: String, // AppImages or Portable Apps
    pub installed: bool,
}

impl PartialEq for AppInfo {
    fn eq(&self, other: &Self) -> bool
    {
        other.name.contains(&self.name) || other.description.contains(&self.description)
    }
}

impl Eq for AppInfo {}

impl PartialOrd for AppInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering>
    {
        Some(self.name.cmp(&other.name))
    }
}

impl Ord for AppInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering
    {
        self.name.cmp(&other.name)
    }
}

pub struct AppWidgets {
    pub name_label: gtk::Label,
    pub description_label: gtk::Label,
    pub db_label: gtk::Label,
    pub action_btn: gtk::Button,
}

impl RelmListItem for AppInfo {
    type Root = gtk::Frame;
    type Widgets = AppWidgets;

    fn setup(_item: &gtk::ListItem) -> (gtk::Frame, AppWidgets)
    {
        relm4::view! {
            app_box = gtk::Frame {
                set_margin_vertical: 1,
                set_expand: false,
                set_hexpand: true,
                set_valign: Align::Start,
                set_vexpand: false,
                set_height_request: 50,

                gtk::Grid {
                    set_column_homogeneous: false,
                    set_row_homogeneous: false,
                    set_margin_horizontal: 8,
                    set_margin_vertical: 12,
                    set_column_spacing: 12,

                    #[name = "name_label"]
                    attach[0, 1, 1, 1] = &gtk::Label {
                        set_halign: Align::Start,
                        add_css_class: "title-4"
                    },

                    attach[0, 0, 1, 1] = &gtk::Image {
                        set_align: Align::Center,
                        set_icon_name: Some("0ad"),
                        set_icon_size: gtk::IconSize::Large,
                        //add_css_class: "title-4"
                    },

                    #[name = "description_label"]
                    attach[1, 0, 1, 1] = &gtk::Label {
                        set_halign: Align::Start,
                        set_xalign: 0.0,
                        set_wrap: true,
                        set_natural_wrap_mode: gtk::NaturalWrapMode::Word,
                        set_hexpand: true,
                        set_width_request: 450,
                        set_lines: 2,
                        add_css_class: "dim-label"
                    },

                    #[name = "action_btn"]
                    attach[2, 0, 1, 1] = &gtk::Button {
                        //set_label: "Install",
                        set_valign: Align::Center,
                        //add_css_class: "suggested-action"
                    },

                    #[name = "db_label"]
                    attach[1, 1, 1, 1] = &gtk::Label {
                        set_halign: Align::Start,
                        //add_css_class: "caption"
                    },
                },
            }
        }

        let widgets = AppWidgets {
            name_label,
            description_label,
            db_label,
            action_btn
        };

        (app_box, widgets)
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root)
    {
        widgets.name_label.set_label(&self.name);
        widgets.description_label.set_label(&self.description);
        widgets.db_label.set_markup(&format!("<b>Database:</b> {}", &self.database));
        if self.installed {
            widgets.action_btn.set_label("Uninstall");
            widgets.action_btn.remove_css_class("suggested-action");
            widgets.action_btn.add_css_class("destructive-action");
        }
        else {
            widgets.action_btn.set_label("Install");
            widgets.action_btn.remove_css_class("destructive-action");
            widgets.action_btn.add_css_class("suggested-action");
        }
    }

    //fn unbind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root)
    //{}
}

//#[derive(Default)]
pub struct App {
    /// Tracks progress status
    searching: bool,
    refresh_sync: bool,

    /// Contains output of a completed task.
    task: Option<CmdOut>,
    /// Holds the apps' widgets
    apps_list_view: TypedListView<AppInfo, gtk::SingleSelection>,
    /// Holds **all** the apps for querying and adding to `apps` if they match
    apps_list: BTreeMap<String, Vec<AppInfo>>,
}

pub struct Widgets {
    errorbox: gtk::Box,
    errorlabel: gtk::Label,
    searchbar: gtk::Entry,
    refreshapps: gtk::Button,
    syncdb: gtk::Button,
    busy_spinner: gtk::Spinner,
    filter_menu: gtk::DropDown,
    //searchbutton: gtk::Button,
}

#[derive(Debug)]
pub enum Input {
    Search(String),
    FetchDatabse,
    RefreshApps,
    UpdateApps(Vec<AppInfo>)
    //InstalledApp(DynamicIndex),
    //UninstalledApp(DynamicIndex),
}

#[derive(Debug)]
pub enum Output {
    Clicked(u32),
}

#[derive(Debug, Default)]
pub enum CmdOut {
    #[default]
    Init,
    SearchDone(Result<Vec<String>, String>),
    RefreshSyncDone,
    GotError(String),
}

impl AsyncComponent for App {
    type Init = ();
    type Input = Input;
    type Output = Output;
    type CommandOutput = CmdOut;
    type Widgets = Widgets;
    type Root = gtk::Window;

    fn init_root() -> Self::Root {
        gtk::Window::builder()
            .title("AppBox")
            .width_request(800)
            .height_request(600)
            .build()
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self>
    {
        let apps_list_view: TypedListView<AppInfo, gtk::SingleSelection> = TypedListView::with_sorting();

        let apps_list: BTreeMap<String, Vec<AppInfo>> = match caching::read_db_apps().await {
            Ok(a) => a,
            Err(e) => {
                sender.command_sender().emit(CmdOut::GotError(e.to_string()));
                BTreeMap::new()
            }
        };

        let model = App {
            searching: false,
            refresh_sync: false,
            task: Some(CmdOut::default()),
            //apps,
            apps_list_view,
            apps_list: apps_list.clone(),
        };

        let apps_view = &model.apps_list_view.view;

        relm4::view! {
            container = gtk::Box {
                //set_halign: Align::Center,
                set_valign: Align::Start,
                set_spacing: 8,
                set_margin_vertical: 14,
                set_margin_horizontal: 100,
                set_orientation: Orientation::Vertical,

                append: errorbox = &gtk::Box {
                    set_hexpand: true,
                    set_vexpand: false,
                    set_align: Align::Fill,
                    set_orientation: Orientation::Horizontal,
                    set_height_request: 200,
                    set_visible: false,

                    gtk::ScrolledWindow {
                        set_expand: true,
                        set_align: Align::Fill,

                        #[name = "errorlabel"]
                        gtk::Label {
                            set_text: "",
                            set_align: Align::Start,
                            set_visible: false,
                        }
                    },
                },

                gtk::Box {
                    set_spacing: 4,
                    set_hexpand: false,
                    set_align: Align::Center,
                    set_orientation: gtk::Orientation::Horizontal,

                    //gtk::Box {
                    //    set_halign: Align::Center,
                    //    set_hexpand: true,
                    //    //set_margin_horizontal: 300,

                    append: searchbar = &gtk::Entry {
                        set_hexpand: false,
                        set_width_request: 700,
                        set_primary_icon_name: Some("system-search-symbolic"),
                        set_secondary_icon_name: Some("edit-clear-symbolic"),
                        set_primary_icon_activatable: true,
                        set_secondary_icon_activatable: true,
                        connect_icon_release[sender] => move |entry, icon_pos| {
                            match icon_pos {
                                gtk::EntryIconPosition::Primary => {
                                    sender.input(Input::Search(entry.text().to_string()));
                                }
                                gtk::EntryIconPosition::Secondary => {
                                    entry.set_text("");
                                }
                                _ => (),
                            }
                        },
                        connect_activate[sender] => move |entry| {
                            sender.input(Input::Search(entry.text().to_string()));
                        },
                    },
                    //},
                },

                gtk::Frame {
                    set_hexpand: true,
                    set_vexpand: false,
                    set_size_request: (500, 50),
                    set_halign: Align::Fill,

                    gtk::Box {
                        set_orientation: Orientation::Horizontal,
                        set_align: Align::Fill,
                        set_expand: false,
                        set_hexpand: true,
                        set_margin_horizontal: 12,

                        gtk::Box {
                            set_orientation: Orientation::Horizontal,
                            set_hexpand: true,
                            set_vexpand: true,
                            set_halign: Align::Start,
                            set_valign: Align::Center,
                            //set_width_request: 400,
                            //set_height_request: 50,
                            //set_margin_horizontal: 12,
                            set_spacing: 8,

                            gtk::Label {
                                set_markup: "<b>Filters:</b>",
                                set_hexpand: false,
                                set_halign: Align::Start,
                            },

                            /*gtk::Box {
                                set_spacing: 10,
                                set_hexpand: true,

                                gtk::CheckButton {
                                    set_label: Some("AppImages"),
                                    set_active: true,
                                    connect_toggled => move |check| {
                                        if check.is_active() {
                                            println!("AppImages checked");
                                        }
                                        else {
                                            println!("AppImages unchecked");
                                        }
                                    }
                                },
                                gtk::CheckButton {
                                    set_label: Some("(Other) Portable Apps"),
                                    set_active: true,
                                    connect_toggled => move |check| {
                                        if check.is_active() {
                                            println!("Portable Apps checked");
                                        }
                                        else {
                                            println!("Portable Apps unchecked");
                                        }
                                    }
                                }
                            },*/

                            append: filter_menu = &gtk::DropDown {}
                        },

                        gtk::Box {
                            set_halign: Align::End,
                            set_hexpand: true,
                            set_vexpand: false,
                            set_spacing: 8,
                            set_margin_all: 12,

                            append: refreshapps = &gtk::Button {
                                set_icon_name: "view-refresh",
                                set_tooltip: "Refresh Apps",
                                connect_clicked[sender] => move |refresh| {
                                    sender.input(Input::RefreshApps);
                                    refresh.set_sensitive(false);
                                }
                            },

                            append: syncdb = &gtk::Button {
                                set_icon_name: "emblem-synchronizing-symbolic",
                                set_tooltip: "Synchronize database",
                                connect_clicked[sender] => move |sync_button| {
                                    sender.input(Input::FetchDatabse);
                                    sync_button.set_sensitive(false);
                                }
                            },
                        },
                    },
                },

                append: busy_spinner = &gtk::Spinner {
                    set_visible: false,
                    set_spinning: false,
                },

                gtk::ScrolledWindow {
                    set_height_request: 450,
                    set_valign: Align::Fill,
                    set_vexpand: true,
                    set_hexpand: true,

                    gtk::Box {
                        set_orientation: Orientation::Vertical,
                        set_margin_horizontal: 16,
                        set_valign: Align::Fill,
                        set_vexpand: true,

                        #[local_ref]
                        apps_view -> gtk::ListView {}
                    },
                },
            }
        }
        let myapps = apps_list.get("AppImages").unwrap();
        sender.input_sender().send(Input::UpdateApps(myapps.to_vec())).unwrap();

        root.set_child(Some(&container));

        AsyncComponentParts {
            model,
            widgets: Widgets {
                errorbox,
                errorlabel,
                searchbar,
                syncdb,
                refreshapps,
                busy_spinner,
                filter_menu,
                //searchbutton,
            },
        }
    }

    async fn update(&mut self, message: Self::Input, sender: AsyncComponentSender<Self>, _root: &Self::Root)
    {
        //let mut apps_guard = self.apps.guard();
        match message {
            Input::FetchDatabse => {
                self.refresh_sync = true;
                sender.command(|out, shutdown| {
                    shutdown.register(async move {
                        println!("before fetch");

                        match caching::purge_db().await {
                            Ok(()) => (),
                            Err(e) => out.send(CmdOut::GotError(e.to_string())).unwrap(),
                        }
                        match caching::create_db().await {
                            Ok(_) => out.send(CmdOut::RefreshSyncDone).unwrap(),
                            Err(e) => out.send(CmdOut::GotError(e.to_string())).unwrap(),
                        };
                        match caching::read_db_apps().await {
                            Ok(r) => util::fs_write(std::path::Path::new("txt"), &format!("{:#?}", r), false, true).unwrap(),
                            Err(e) => out.send(CmdOut::GotError(e.to_string())).unwrap(),
                        }

                        println!("after fetch");
                    }).drop_on_shutdown().boxed()
                });
            }
            Input::RefreshApps => {}
            Input::Search(query) => {
                self.searching = true;
                sender.command(|out, shutdown| {
                    shutdown
                        // Performs this operation until a shutdown is triggered
                        .register(async move {
                            println!("{}", &query);
                            let query = query.trim();

                            if query.is_empty() {
                                out.send(CmdOut::SearchDone(Ok(Vec::new()))).unwrap();
                                return;
                            }
                            let dummy: Vec<String> = vec![
                                String::from("test"),
                                String::from("daav"),
                                String::from("gdgd"),
                                String::from("bcbx"),
                                String::from("uotqr"),
                            ];

                            let mut results: Vec<String> = Vec::new();
                            for entry in &dummy {
                                if entry.contains(&query) {
                                    results.push(entry.to_string());
                                }
                            }

                            out.send(CmdOut::SearchDone(Ok(results))).unwrap();
                        })
                        // Perform task until a shutdown interrupts it
                        .drop_on_shutdown()
                        // Wrap into a `Pin<Box<Future>>` for return
                        .boxed()
                });
            }
            Input::UpdateApps(apps) => {
                self.apps_list_view.clear();
                for app in apps {
                    self.apps_list_view.append(app);
                }
            }
            //Input::InstalledApp(_index) => {()}
            //Input::UninstalledApp(_index) => {()}
        }
    }

    async fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    )
    {
        if let CmdOut::SearchDone(_) = message {
            self.searching = false;
        }
        match message {
            CmdOut::SearchDone(_) => self.searching = false,
            CmdOut::RefreshSyncDone => self.refresh_sync = false,
            _ => (),
        }

        self.task = Some(message);
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: AsyncComponentSender<Self>) {
        //widgets.searchbutton.set_sensitive(!self.searching);
        widgets.searchbar.set_editable(!self.searching);

        widgets.refreshapps.set_sensitive(!self.refresh_sync);
        widgets.syncdb.set_sensitive(!self.refresh_sync);
        widgets.busy_spinner.set_spinning(self.refresh_sync);
        widgets.busy_spinner.set_visible(self.refresh_sync);

        if let Some(ref progress) = self.task {
            match progress {
                CmdOut::Init => (),
                CmdOut::SearchDone(result) => {
                    match result {
                        Ok(results) => {
                            println!("{:#?}", results);
                        }
                        Err(e) => eprintln!("{e}"),
                    }
                }
                /*CmdOut::RefreshSyncDone => {
                    widgets.syncdb.set_sensitive(true);
                    widgets.busy_spinner.set_spinning(false);
                    widgets.busy_spinner.set_visible(false);
                }
                CmdOut::RefreshDone => {
                    widgets.refreshapps.set_sensitive(true);
                    widgets.busy_spinner.set_spinning(false);
                    widgets.busy_spinner.set_visible(false);
                }*/
                CmdOut::GotError(what) => {
                    widgets.errorlabel.set_text(what);
                    widgets.errorlabel.set_visible(true);
                    widgets.errorbox.set_visible(true);
                }
                // to always have "uncovered arm" warning
                CmdOut::RefreshSyncDone => (),
            }
        }
    }
}
