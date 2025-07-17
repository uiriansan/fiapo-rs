mod imp {
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::{Application, ApplicationWindow, glib};

    use crate::core::utils::styles;

    #[derive(Default)]
    pub struct App {}

    #[glib::object_subclass]
    impl ObjectSubclass for App {
        const NAME: &'static str = "MyApplication";
        type Type = super::App;
        type ParentType = Application;
    }

    impl ObjectImpl for App {}

    impl ApplicationImpl for App {
        fn activate(&self) {
            let application = self.obj();

            const CSS_FILE: &str = "styles/default.css";
            styles::load_css(CSS_FILE);

            let window = ApplicationWindow::builder()
                .application(&*application)
                .title("My GTK4 App")
                .default_width(350)
                .default_height(250)
                .build();

            let button = gtk4::Button::with_label("Click me!");
            button.connect_clicked(|_| {
                println!("Button clicked!");
            });

            window.set_child(Some(&button));
            window.present();
        }

        fn startup(&self) {
            self.parent_startup();
            println!("Application starting up...");
        }
    }

    impl GtkApplicationImpl for App {}
}

use gtk4::{Application, glib};

glib::wrapper! {
    pub struct App(ObjectSubclass<imp::App>)
        @extends Application, gtk4::gio::Application,
        @implements gtk4::gio::ActionGroup, gtk4::gio::ActionMap;
}

impl App {
    pub fn new(application_id: &str, flags: gtk4::gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build()
    }
}
