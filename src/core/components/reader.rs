use crate::core::app::FiapoController;
use glib::clone;
use gtk::prelude::{ButtonExt, OrientableExt, WidgetExt};
use gtk::{CenterBox, Picture, gdk, gdk_pixbuf, glib};
use gtk4::gdk::{Key, ModifierType};
use gtk4::{self as gtk, EventControllerKey};
use image::DynamicImage;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Reader {
    controller: Rc<RefCell<FiapoController>>,
    container: CenterBox,
    picture: Picture,
}
impl Reader {
    pub fn new(controller: Rc<RefCell<FiapoController>>) -> Self {
        let container = CenterBox::new();
        container.set_orientation(gtk::Orientation::Vertical);
        let picture = Picture::new();
        Self {
            controller,
            container,
            picture,
        }
    }

    pub fn build(&mut self) -> CenterBox {
        let label = gtk::Label::new(Some("Reader"));
        let btn = gtk::Button::with_label("<- Back");
        btn.connect_clicked(clone!(
            #[strong(rename_to = controller)]
            self.controller,
            move |_| controller.borrow_mut().go_home()
        ));

        let container = CenterBox::new();
        container.set_orientation(gtk::Orientation::Horizontal);
        container.set_start_widget(Some(&label));
        container.set_end_widget(Some(&btn));

        self.next_page();

        self.container.set_start_widget(Some(&container));
        self.container.set_center_widget(Some(&self.picture));

        let key_handler = gtk::EventControllerKey::new();
        let window = self.controller.borrow().window.clone();
        {
            let mut s = Rc::new(&self);
            key_handler.connect_key_pressed(move |_, key, _, _| s.handle_key_press(key));
        }
        window.add_controller(key_handler);

        self.container.clone()
    }

    fn handle_key_press(&mut self, key: Key) -> gtk::glib::Propagation {
        match key {
            gtk::gdk::Key::Left => self.next_page(),
            gtk::gdk::Key::Right => self.prev_page(),
            _ => println!("{key}"),
        }
        gtk::glib::Propagation::Stop
    }

    fn prev_page(&mut self) {
        match self.controller.borrow_mut().server.get_prev_page() {
            Some(page) => {
                if let Ok(texture) = self.dynamic_image_to_texture(page) {
                    self.picture.set_paintable(Some(&texture));
                }
            }
            _ => {}
        }
    }

    fn next_page(&mut self) {
        match self.controller.borrow_mut().server.get_next_page() {
            Some(page) => {
                if let Ok(texture) = self.dynamic_image_to_texture(page) {
                    self.picture.set_paintable(Some(&texture));
                }
            }
            _ => {}
        }
    }

    fn dynamic_image_to_pixbuf(
        &self,
        img: &DynamicImage,
    ) -> Result<gdk_pixbuf::Pixbuf, glib::Error> {
        let rgba_img = img.to_rgba8();
        let (width, height) = rgba_img.dimensions();
        let bytes = rgba_img.into_raw();
        let pixbuf = gdk_pixbuf::Pixbuf::from_bytes(
            &glib::Bytes::from(&bytes),
            gdk_pixbuf::Colorspace::Rgb,
            true, // alpha
            8,    // bits per pixel
            width as i32,
            height as i32,
            (width * 4) as i32,
        );
        Ok(pixbuf)
    }

    // Convert a DynamicImage from the Image crate to Gdk.Texture
    pub fn dynamic_image_to_texture(
        &self,
        img: &DynamicImage,
    ) -> Result<gdk::Texture, glib::Error> {
        let pixbuf = self.dynamic_image_to_pixbuf(&img).unwrap();
        let texture = gdk::Texture::for_pixbuf(&pixbuf);
        Ok(texture)
    }
}
