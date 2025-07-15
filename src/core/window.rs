use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use gtk4 as gtk;

use crate::core::components::icon_button;

pub fn create(app: Application) {
    let window = ApplicationWindow::builder()
        .application(&app)
        .title("Fiapo")
        .default_width(800)
        .default_height(600)
        .build();

    let button = icon_button::create("Click me!!!");
    button.connect_clicked(|_| {
        eprintln!("Clicked!");
    });
    window.set_child(Some(&button));
    window.present();
}
