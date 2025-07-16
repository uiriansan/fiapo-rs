use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Image};
use gtk4 as gtk;

use crate::core::components::icon_button;
use crate::core::utils::image;

use pdf2image::{PDF, PDF2ImageError, RenderOptionsBuilder};

fn img_test(window: &ApplicationWindow) {
    let container = Box::new(gtk::Orientation::Vertical, 10);
    container.set_margin_top(200);

    let img = image::open_image("assets/wha.jpg").unwrap();
    let texture = image::dynamic_image_to_texture(&img);

    let image = Image::from_paintable(Some(&texture.unwrap()));
    image.set_size_request(0, 500);
    image.set_margin_top(50);

    let img_rc = Rc::new(RefCell::new(img));
    let cloned_img = img_rc.clone();
    let cloned_gtk_img = image.clone();

    let button = icon_button::create("Click me!!!");
    button.connect_clicked(move |_| {
        let mut img = cloned_img.borrow_mut();
        img.invert();
        let texture = image::dynamic_image_to_texture(&img);
        cloned_gtk_img.set_paintable(Some(&texture.unwrap()));
    });
    button.set_cursor(gtk4::gdk::Cursor::from_name("pointer", None).as_ref());

    container.append(&button);
    container.append(&image);

    window.set_child(Some(&container));
}

fn pdf_test(window: &ApplicationWindow) -> Result<(), PDF2ImageError> {
    let container = Box::new(gtk::Orientation::Vertical, 10);
    container.set_margin_top(200);

    let pdf_file =
        PDF::from_file("/home/uirian/Downloads/Houseki no Kuni/HOUSEKI NO KUNI VOL.01.pdf")
            .unwrap();
    let pages = pdf_file.render(
        pdf2image::Pages::Range(1..=5),
        RenderOptionsBuilder::default().pdftocairo(true).build()?,
    )?;

    let img = pages[4].clone();
    let texture = image::dynamic_image_to_texture(&img);
    let image = Image::from_paintable(Some(&texture.unwrap()));
    image.set_size_request(0, 500);

    let img_rc = Rc::new(RefCell::new(img));
    let cloned_img = img_rc.clone();
    let cloned_gtk_img = image.clone();

    let button = icon_button::create("Click me!!!");
    button.connect_clicked(move |_| {
        let mut img = cloned_img.borrow_mut();
        img.invert();
        let texture = image::dynamic_image_to_texture(&img);
        cloned_gtk_img.set_paintable(Some(&texture.unwrap()));
    });
    button.set_cursor(gtk4::gdk::Cursor::from_name("pointer", None).as_ref());

    container.append(&button);
    container.append(&image);

    window.set_child(Some(&container));
    Ok(())
}

pub fn create(app: Application) {
    let window = ApplicationWindow::builder()
        .application(&app)
        .title("Fiapo")
        .default_width(800)
        .default_height(600)
        .build();

    // img_test(&window);
    let _ = pdf_test(&window);
    window.present();
}
