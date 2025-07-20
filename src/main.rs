use gtk::prelude::*;
use gtk::{Application, glib};
use gtk4 as gtk;

mod core;
use core::app::AppContext;

use pdfium_render::prelude::*;

const CSS_FILE: &str = "styles/default.css";

fn main() -> glib::ExitCode {
    let application = Application::builder()
        .application_id("github.uiriansan.fiapo")
        .build();

    application.connect_activate(|app| {
        let app_context = AppContext::new(&app);
        app_context.load_css(CSS_FILE);
        app_context.setup_home();
    });

    application.run()
}
