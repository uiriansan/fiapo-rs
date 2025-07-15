use gtk::prelude::*;
use gtk::{Application, glib};
use gtk4 as gtk;

mod core;
use core::utils::image;
use core::window;

fn main() -> glib::ExitCode {
    let application = Application::builder()
        .application_id("com.uiriansan.fiapo")
        .build();

    application.connect_activate(|app| {
        window::create(app.clone());
    });

    let _ = image::invert_image();

    application.run()
}
