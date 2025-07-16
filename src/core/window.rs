use std::cell::RefCell;
use std::rc::Rc;

use ::image::DynamicImage;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Picture, ProgressBar};
use gtk4 as gtk;
use gtk4::glib::MainContext;

use crate::core::components::icon_button;
use crate::core::utils::image;

use pdf2image::{PDF, RenderOptionsBuilder};

#[allow(dead_code)]
struct ReaderState {
    image_component: Picture,
    page_label: gtk::Label,
    progress_bar: ProgressBar,
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
        progress_bar: &ProgressBar,
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

        progress_bar.set_fraction(1.0 / page_count as f64);
        page_label.set_text(("1".to_string() + "/" + &page_count.to_string()).as_str());

        Ok(ReaderState {
            image_component: image_component.to_owned(),
            page_label: page_label.to_owned(),
            progress_bar: progress_bar.to_owned(),
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

        self.update_progress_bar();
        self.update_label();
    }

    fn update_progress_bar(&mut self) {
        let current_page = 1.0 + self.current_page_index as f64;
        let page_count = self.page_count as f64;

        self.progress_bar.set_fraction(current_page / page_count);
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

    fn first_page(&mut self) {
        self.current_page_index = 0;
        self.update_page();
    }

    fn last_page(&mut self) {
        self.current_page_index = self.page_count - 1;
        self.update_page();
    }

    fn toggle_invert_image_colors(&mut self) {
        self.invert = !self.invert;
        self.update_page();
    }
}

async fn open_file(window: &gtk::ApplicationWindow) {
    let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
    let file_filter = gtk::FileFilter::new();
    file_filter.add_suffix("pdf");
    // file_filter.add_suffix("png");
    // file_filter.add_suffix("jpg");
    filters.append(&file_filter);

    let file_dialog = gtk::FileDialog::builder()
        .title("Open file or directory")
        .accept_label("Open")
        .modal(true)
        .filters(&filters)
        .build();

    if let Ok(file) = file_dialog.open_future(Some(window)).await {
        render_pdf(&window, file);
    }
}

fn show_home(window: Rc<ApplicationWindow>) {
    let btn_open = icon_button::create("Open");
    btn_open.set_cursor(gtk4::gdk::Cursor::from_name("pointer", None).as_ref());
    {
        let window = Rc::clone(&window);
        btn_open.connect_clicked(move |_| {
            let window = Rc::clone(&window);
            MainContext::default().spawn_local(async move {
                open_file(&window).await;
            });
        });
    }
    window.set_child(Some(&btn_open));
}

fn render_pdf(window: &ApplicationWindow, file: gtk::gio::File) {
    let container = gtk::CenterBox::new();
    container.set_orientation(gtk::Orientation::Vertical);
    container.set_margin_top(25);

    let picture = Picture::new();
    let label = gtk::Label::new(Some(""));
    let progress_bar = ProgressBar::builder()
        .hexpand(true)
        .show_text(false)
        .sensitive(false)
        .build();
    let reader_state = Rc::new(RefCell::new(
        ReaderState::new(
            file.path().unwrap().to_str().unwrap(),
            &picture,
            &label,
            &progress_bar,
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

    let header = gtk::CenterBox::new();
    header.set_orientation(gtk::Orientation::Horizontal);
    header.set_margin_start(100);
    header.set_margin_end(100);
    header.set_margin_bottom(25);
    header.set_start_widget(Some(&label));
    let control_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    control_box.append(&btn_prev);
    control_box.append(&btn_invert);
    control_box.append(&btn_next);
    header.set_end_widget(Some(&control_box));
    container.set_start_widget(Some(&header));
    container.set_center_widget(Some(&picture));
    container.set_end_widget(Some(&progress_bar));

    window.set_child(Some(&container));

    let key_handler = gtk::EventControllerKey::new();
    key_handler.connect_key_pressed(move |_, key, _, _| {
        match key {
            gtk::gdk::Key::Left => {
                let reader_state = Rc::clone(&reader_state);
                reader_state.borrow_mut().prev_page();
            }
            gtk::gdk::Key::Right => {
                let reader_state = Rc::clone(&reader_state);
                reader_state.borrow_mut().next_page();
            }
            gtk::gdk::Key::i => {
                let reader_state = Rc::clone(&reader_state);
                reader_state.borrow_mut().toggle_invert_image_colors();
            }
            gtk::gdk::Key::f => {
                header.set_visible(!header.get_visible());
                let mut container_margin = container.margin_top();
                if container_margin == 0 {
                    container_margin = 25;
                } else {
                    container_margin = 0;
                }
                container.set_margin_top(container_margin);
            }
            gtk::gdk::Key::End => {
                let reader_state = Rc::clone(&reader_state);
                reader_state.borrow_mut().last_page();
            }
            gtk::gdk::Key::Home => {
                let reader_state = Rc::clone(&reader_state);
                reader_state.borrow_mut().first_page();
            }
            _ => println!("{key}"),
        }
        gtk::glib::Propagation::Stop
    });
    window.add_controller(key_handler);
}

pub fn create(app: Application) {
    let window = Rc::new(
        ApplicationWindow::builder()
            .application(&app)
            .title("Fiapo")
            .default_width(800)
            .default_height(600)
            .css_classes(["main-window"])
            .build(),
    );

    show_home(Rc::clone(&window));

    window.present();
}
