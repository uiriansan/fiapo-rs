use crate::core::app::FiapoController;
use glib::clone;
use gtk::prelude::{ButtonExt, OrientableExt};
use gtk::{CenterBox, Picture, gdk, gdk_pixbuf, glib};
use gtk4 as gtk;
use image::DynamicImage;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Reader {
    _controller: Rc<RefCell<FiapoController>>,
    _container: CenterBox,
}
impl Reader {
    pub fn new(controller: Rc<RefCell<FiapoController>>) -> Self {
        let container = CenterBox::new();
        container.set_orientation(gtk::Orientation::Vertical);
        Self {
            _controller: controller,
            _container: container,
        }
    }

    pub fn build(&self) -> CenterBox {
        let label = gtk::Label::new(Some("Reader"));
        let btn = gtk::Button::with_label("<- Back");
        btn.connect_clicked(clone!(
            #[strong(rename_to = controller)]
            self._controller,
            move |_| controller.borrow_mut().go_home()
        ));

        let picture = Picture::new();
        let container = CenterBox::new();
        container.set_orientation(gtk::Orientation::Horizontal);
        container.set_start_widget(Some(&label));
        container.set_end_widget(Some(&btn));

        match self._controller.borrow_mut().server.get_next_page() {
            Some(page) => {
                if let Ok(texture) = self.dynamic_image_to_texture(page) {
                    picture.set_paintable(Some(&texture));
                }
            }
            _ => {}
        }

        self._container.set_start_widget(Some(&container));
        self._container.set_center_widget(Some(&picture));

        self._container.clone()
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
