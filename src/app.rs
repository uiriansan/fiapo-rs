use crate::core::config::{FiapoConfig, resolve_config_path};
use crate::core::reader::Server;
use crate::ui::home::Home;
use crate::ui::reader::Reader;
use gtk::prelude::GtkWindowExt;
use gtk::{Application, ApplicationWindow, Stack, gdk, gio};
use gtk4 as gtk;
use log::{error, info};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct FiapoController {
    pub config: FiapoConfig,
    pub config_path: Option<PathBuf>,
    pub window: ApplicationWindow,
    pub view_stack: Stack,
    pub server: Server,
}

impl FiapoController {
    pub fn new(app: &Application) -> Self {
        let config = FiapoConfig::defaults();

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Fiapo")
            .default_width(1280)
            .default_height(720)
            .css_name("fiapo-window")
            .decorated(false)
            .build();
        let view_stack = Stack::new();
        view_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        view_stack.set_transition_duration(250);
        view_stack.set_interpolate_size(true);
        let server = Server::new();

        Self {
            config,
            config_path: None,
            window,
            view_stack,
            server: server,
        }
    }

    pub fn build_ui(controller: Rc<RefCell<FiapoController>>) {
        let window = controller.borrow_mut().window.clone();
        let stack = controller.borrow_mut().view_stack.clone();

        {
            let controller = Rc::clone(&controller);
            let home = Home::new(controller);
            let home_screen = home.build();

            stack.add_named(&home_screen, Some("home_screen"));
            stack.set_visible_child_name("home_screen");

            window.set_child(Some(&stack));
            window.present();
        }

        // debug!("{:?}", controller.borrow());
        // debug!("");
    }

    pub fn go_home(&self) {
        self.view_stack.set_visible_child_name("home_screen");
    }
    pub fn open_reader(controller: Rc<RefCell<FiapoController>>) {
        if let Some(old_reader) = controller
            .borrow()
            .view_stack
            .child_by_name("reader_screen")
        {
            controller.borrow().view_stack.remove(&old_reader);
        }

        let stack = controller.borrow_mut().view_stack.clone();
        let reader = Rc::new(RefCell::new(Reader::new(controller)));
        let reader_screen = Reader::build(reader);
        stack.add_named(&reader_screen, Some("reader_screen"));
        stack.set_visible_child_name("reader_screen");
    }

    pub fn get_window(&self) -> ApplicationWindow {
        self.window.clone()
    }

    pub fn load_config(&mut self, path: &str) {
        let resolve_path = resolve_config_path(path);
        self.config_path = resolve_path.clone();
        match resolve_path {
            Some(config_path) => self.config.parse_config_file(config_path),
            _ => {}
        }
    }

    pub fn load_css(&self, file_path: &str) {
        if let Some(display) = gdk::Display::default() {
            let entry_path = PathBuf::from(file!());
            if let Some(srcdir) = entry_path.parent() {
                // 'canonicalize' might be overkill, but...
                if let Ok(working_dir) = fs::canonicalize(&srcdir) {
                    let css_path = working_dir.join(file_path);
                    let provider = gtk::CssProvider::new();
                    provider.load_from_file(&gio::File::for_path(&css_path));
                    gtk::style_context_add_provider_for_display(
                        &display,
                        &provider,
                        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                    );
                    info!("Loading styles from {}...", css_path.to_str().unwrap());
                } else {
                    error!("Failed to resolve path for styles");
                }
            } else {
                error!("Failed to resolve path for styles");
            }
        } else {
            error!("Could not retrieve default `Gdk.Display`");
        }
    }
}
