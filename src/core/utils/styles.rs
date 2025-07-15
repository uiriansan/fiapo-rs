use gtk4 as gtk;

pub fn load_css(file: &str) {
    let display = gtk::gdk::Display::default().expect("Could not get default display!");
    let provider = gtk::CssProvider::new();
    provider.load_from_file(&gtk::gio::File::for_path(file));
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
