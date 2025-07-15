use gtk::Button;
use gtk4 as gtk;

pub fn create(label: &str) -> Button {
    return Button::with_label(label);
}
