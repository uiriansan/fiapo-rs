use crate::core::app::FiapoController;
use crate::core::server::Server;
use glib::clone;
use gtk::gdk::Key;
use gtk::prelude::{ButtonExt, OrientableExt, WidgetExt};
use gtk::{CenterBox, Picture, gdk, gdk_pixbuf, glib};
use gtk4 as gtk;
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

    pub fn build(reader: Rc<RefCell<Self>>) -> CenterBox {
        let label = gtk::Label::new(Some("Reader"));
        let btn = gtk::Button::with_label("<- Back");
        btn.connect_clicked(clone!(
            #[strong(rename_to = controller)]
            reader.borrow().controller,
            move |_| controller.borrow_mut().go_home()
        ));

        let container = CenterBox::new();
        container.set_orientation(gtk::Orientation::Horizontal);
        container.set_start_widget(Some(&label));
        container.set_end_widget(Some(&btn));

        reader.borrow_mut().next_page();

        reader.borrow().container.set_start_widget(Some(&container));
        reader
            .borrow()
            .container
            .set_center_widget(Some(&reader.borrow().picture));

        let key_handler = gtk::EventControllerKey::new();
        let window = reader.borrow().controller.borrow().window.clone();

        key_handler.connect_key_pressed(clone!(
            #[strong]
            reader,
            move |_, key, _, _| {
                match key {
                    Key::Left => reader.borrow_mut().next_page(),
                    Key::Right => reader.borrow_mut().prev_page(),
                    _ => println!("{key}"),
                }
                gtk::glib::Propagation::Stop
            }
        ));
        window.add_controller(key_handler);

        reader.borrow().container.clone()
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
