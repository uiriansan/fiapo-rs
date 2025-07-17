use gtk::prelude::*;
use gtk::{gio, glib};
use gtk4 as gtk;

mod core;
use core::app::App;

fn main() -> glib::ExitCode {
    let app = App::new("github.uiriansan.fiapo", gio::ApplicationFlags::empty());
    app.run()
}
