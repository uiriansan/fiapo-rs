use gtk::prelude::*;
use gtk::{Application, glib};
use gtk4 as gtk;

mod core;
use core::utils::{image, styles};
use core::window;

fn main() -> glib::ExitCode {
    let application = Application::builder()
        .application_id("com.uiriansan.fiapo")
        .build();

    let css_file = "styles/default.css";

    application.connect_activate(|app| {
        window::create(app.clone());
        styles::load_css(css_file);
    });

    let _ = image::invert_image();

    application.run()
}
