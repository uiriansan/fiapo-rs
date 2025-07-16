use std::cell::RefCell;
use std::rc::Rc;

use ::image::DynamicImage;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Image, Picture};
use gtk4 as gtk;

use crate::core::components::icon_button;
use crate::core::utils::image;

use pdf2image::{PDF, RenderOptionsBuilder};

struct ReaderState {
    image_component: Picture,
    page_label: gtk::Label,
    pdf_file: PDF,
    pdf_file_path: String,
    page_count: usize,
    current_page_index: usize,
    current_page_img: DynamicImage,
    invert: bool,
}

impl ReaderState {
    fn new(
        pdf_file_path: &str,
        image_component: &Picture,
        page_label: &gtk::Label,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let pdf_file = PDF::from_file(&pdf_file_path)
            .expect(format!("Could not open file `{pdf_file_path}`").as_str());
        let page = pdf_file.render(
            pdf2image::Pages::Single(1),
            RenderOptionsBuilder::default().pdftocairo(true).build()?,
        )?;
        let page_count = pdf_file.page_count();

        let texture = image::dynamic_image_to_texture(&page[0].clone());
        image_component.set_paintable(Some(&texture.unwrap()));

        page_label.set_text(("1".to_string() + "/" + &page_count.to_string()).as_str());

        Ok(ReaderState {
            image_component: image_component.to_owned(),
            page_label: page_label.to_owned(),
            pdf_file: pdf_file,
            pdf_file_path: pdf_file_path.to_owned(),
            page_count: page_count as usize,
            current_page_index: 0,
            current_page_img: page[0].clone(),
            invert: false,
        })
    }

    fn get_page_at_index(
        &mut self,
        index: Option<usize>,
    ) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        let uindex = index.unwrap_or(self.current_page_index);
        let str_index = &uindex.to_string();
        let page = self
            .pdf_file
            .render(
                pdf2image::Pages::Single(1 + uindex as u32),
                RenderOptionsBuilder::default().pdftocairo(true).build()?,
            )
            .expect(format!("Could not load page `{str_index}`").as_str());

        Ok(page[0].clone())
    }

    fn update_page(&mut self) {
        self.current_page_img = self.get_page_at_index(None).unwrap();

        if self.invert {
            self.current_page_img.invert();
        }

        let texture = image::dynamic_image_to_texture(&self.current_page_img);
        self.image_component.set_paintable(Some(&texture.unwrap()));

        self.update_label();
    }

    fn update_label(&mut self) {
        let current_page = (self.current_page_index + 1).to_string();
        let page_count = self.page_count.to_string();

        self.page_label
            .set_text((current_page + "/" + &page_count).as_str());
    }

    fn next_page(&mut self) {
        let next_index = (self.current_page_index + self.page_count + 1) % self.page_count;
        self.current_page_index = next_index;
        self.update_page();
    }

    fn prev_page(&mut self) {
        let prev_index = (self.current_page_index + self.page_count - 1) % self.page_count;
        self.current_page_index = prev_index;
        self.update_page();
    }

    fn toggle_invert_image_colors(&mut self) {
        self.invert = !self.invert;
        self.update_page();
    }
}

fn img_test(window: &ApplicationWindow) {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 10);
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

fn pdf_test(window: &ApplicationWindow) {
    let container = gtk::Box::new(gtk::Orientation::Vertical, 10);
    container.set_margin_top(25);

    let picture = Picture::new();
    let label = gtk::Label::new(Some(""));
    let reader_state = Rc::new(RefCell::new(
        ReaderState::new(
            "/home/uirian/Downloads/Houseki no Kuni/HOUSEKI NO KUNI VOL.01.pdf",
            &picture,
            &label,
        )
        .unwrap(),
    ));

    let btn_prev = icon_button::create(" < ");
    btn_prev.set_cursor(gtk4::gdk::Cursor::from_name("pointer", None).as_ref());
    {
        let reader_state = Rc::clone(&reader_state);
        btn_prev.connect_clicked(move |_| reader_state.borrow_mut().prev_page());
    }

    let btn_invert = icon_button::create("Invert");
    btn_invert.set_cursor(gtk4::gdk::Cursor::from_name("pointer", None).as_ref());
    {
        let reader_state = Rc::clone(&reader_state);
        btn_invert.connect_clicked(move |_| reader_state.borrow_mut().toggle_invert_image_colors());
    }

    let btn_next = icon_button::create(" > ");
    btn_next.set_cursor(gtk4::gdk::Cursor::from_name("pointer", None).as_ref());
    {
        let reader_state = Rc::clone(&reader_state);
        btn_next.connect_clicked(move |_| reader_state.borrow_mut().next_page());
    }

    let mouse_gesture_controller = gtk::GestureClick::new();
    container.add_controller(mouse_gesture_controller.clone());
    mouse_gesture_controller.set_button(0);
    {
        let reader_state = Rc::clone(&reader_state);
        mouse_gesture_controller.connect_released(move |gesture, _, _, _| {
            if gesture.current_button() == 1 {
                // left mouse button
                reader_state.borrow_mut().prev_page();
            } else if gesture.current_button() == 3 {
                // right mouse button
                reader_state.borrow_mut().next_page();
            }
        });
    }

    let scroll_mouse_controller =
        gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
    container.add_controller(scroll_mouse_controller.clone());
    {
        let reader_state = Rc::clone(&reader_state);
        scroll_mouse_controller.connect_scroll(move |_, _, direction| {
            if direction == 1.0 {
                // scroll down
                reader_state.borrow_mut().next_page();
            } else if direction == -1.0 {
                // scroll up
                reader_state.borrow_mut().prev_page();
            }
            gtk::glib::Propagation::Stop
        });
    }

    let label_center_box = gtk::CenterBox::new();
    label_center_box.set_center_widget(Some(&label));

    let center_box = gtk::CenterBox::new();
    let row_container = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row_container.append(&btn_prev);
    row_container.append(&btn_invert);
    row_container.append(&btn_next);
    center_box.set_center_widget(Some(&row_container));
    container.append(&label_center_box);
    container.append(&center_box);
    container.append(&picture);

    window.set_child(Some(&container));

    let key_handler = gtk::EventControllerKey::new();
    key_handler.connect_key_pressed(move |_, _, c, _| {
        if c == 113 {
            // left arrow
            let reader_state = Rc::clone(&reader_state);
            reader_state.borrow_mut().prev_page();
        } else if c == 114 {
            // right arrow
            let reader_state = Rc::clone(&reader_state);
            reader_state.borrow_mut().next_page();
        }
        gtk::glib::Propagation::Stop
    });
    window.add_controller(key_handler);
}

pub fn create(app: Application) {
    let window = ApplicationWindow::builder()
        .application(&app)
        .title("Fiapo")
        .default_width(800)
        .default_height(600)
        .build();

    // img_test(&window);
    pdf_test(&window);

    window.present();
}
