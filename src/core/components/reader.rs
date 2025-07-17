use image::DynamicImage;

pub struct ReaderState {
    source: String,
    page_count: usize,
    pages: Vec<DynamicImage>,
    current_page_index: usize,
    // invert: bool,
}

impl ReaderState {
    pub fn new(source: &str, pages: Vec<DynamicImage>, page_count: usize) -> Self {
        ReaderState {
            source: source.to_string(),
            page_count: page_count,
            pages: pages,
            current_page_index: 0,
        }
    }

    pub fn next_page(&mut self) {}

    pub fn prev_page(&mut self) {}

    pub fn go_to_page(&mut self, page: usize) {
        let _page = page - 1; // page index
    }

    fn load_pdf(&mut self) {
        // load pdf...
        // convert to Vec<DynamicImage>
        // self.pages
    }
}

mod imp {
    use gtk::subclass::prelude::*;
    use gtk::{Box, glib};
    use gtk4 as gtk;

    #[derive(Default)]
    pub struct Reader {}

    #[glib::object_subclass]
    impl ObjectSubclass for Reader {
        const NAME: &str = "FiapoReader";
        type Type = super::Reader;
        type ParentType = Box;
    }

    impl ObjectImpl for Reader {}
    impl BoxImpl for Reader {}
    impl WidgetImpl for Reader {}
}

use crate::core::app::AppContext;
use gtk::Box;
use gtk::prelude::*;
use gtk::{Widget, glib};
use gtk4 as gtk;
use std::cell::RefCell;
use std::rc::Rc;

glib::wrapper! {
    pub struct Reader(ObjectSubclass<imp::Reader>)
        @extends Box, Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Reader {
    pub fn new(_app_context: &Rc<RefCell<AppContext>>) -> Self {
        let reader: Reader = glib::Object::builder()
            .property("orientation", gtk::Orientation::Vertical)
            .property("spacing", 0)
            .build();
        reader.set_hexpand(true);
        reader.set_vexpand(true);
        // reader.append(&btn_open);
        return reader;
    }
}
