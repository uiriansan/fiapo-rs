mod imp {
    use gtk::subclass::prelude::*;
    use gtk::{Box, glib};
    use gtk4 as gtk;

    #[derive(Default)]
    pub struct Home {}

    #[glib::object_subclass]
    impl ObjectSubclass for Home {
        const NAME: &'static str = "FiapoHome";
        type Type = super::Home;
        type ParentType = Box;
    }

    impl ObjectImpl for Home {}
    impl BoxImpl for Home {}
    impl WidgetImpl for Home {}
}

use crate::core::app::AppContext;
use crate::core::components::icon_button;
use glib::clone;
use gtk::Box;
use gtk::prelude::*;
use gtk::{Widget, glib};
use gtk4 as gtk;
use gtk4::glib::MainContext;
use std::cell::RefCell;
use std::rc::Rc;

glib::wrapper! {
    pub struct Home(ObjectSubclass<imp::Home>)
        @extends Box, Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

async fn open_file_dialog(window: &gtk::ApplicationWindow) -> Result<gtk::gio::File, glib::Error> {
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

    file_dialog.open_future(Some(window)).await
}

impl Home {
    pub fn new(app_context: &Rc<RefCell<AppContext>>) -> Self {
        let btn_open = icon_button::create("Open");
        btn_open.set_cursor(gtk4::gdk::Cursor::from_name("pointer", None).as_ref());
        btn_open.connect_clicked(clone!(
            #[strong]
            app_context,
            move |_| {
                MainContext::default().spawn_local({
                    let app_context = Rc::clone(&app_context);
                    async move {
                        let window = {
                            let ctx = app_context.borrow();
                            ctx.get_window()
                        };

                        match open_file_dialog(&window).await {
                            Ok(file) => {
                                let path = file.path().unwrap().to_str().unwrap().to_string();

                                AppContext::open_reader(app_context.clone(), &path).unwrap();
                            }
                            Err(e) => println!("Could not open file: {}", e),
                        };
                    }
                });
            }
        ));

        let home: Home = glib::Object::builder()
            .property("orientation", gtk::Orientation::Vertical)
            .property("spacing", 0)
            .build();
        home.set_hexpand(true);
        home.set_vexpand(true);
        home.append(&btn_open);
        return home;
    }
}
