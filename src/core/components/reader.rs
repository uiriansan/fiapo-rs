use crate::core::app::FiapoController;
use glib::clone;
use gtk::{CenterBox, glib};
use gtk4 as gtk;
use gtk4::prelude::{BoxExt, ButtonExt};
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

        let container = gtk::Box::new(gtk::Orientation::Vertical, 10);
        container.append(&label);
        container.append(&btn);

        self._container.set_center_widget(Some(&container));

        self._container.clone()
    }
}
