use gtk::{Application, ApplicationWindow, Stack};
use gtk4::{self as gtk, prelude::GtkWindowExt};
use pdf2image::{PDF, RenderOptionsBuilder};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use crate::core::components::{
    home::Home,
    reader::{Reader, ReaderState},
};
use crate::core::server::server::Server;

pub struct AppContext {
    window: ApplicationWindow,
    view_stack: Stack,
    server: Server,
    reader_state: Option<ReaderState>,
}
impl AppContext {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Fiapo")
            .default_width(800)
            .default_height(600)
            .css_classes(["fiapo-window"])
            .decorated(false)
            .build();
        let view_stack = Stack::new();

        window.present();

        let server = Server::new();

        Self {
            window,
            view_stack,
            server,
            reader_state: None,
        }
    }

    pub fn setup_home(self) {
        let context = Rc::new(RefCell::new(self));
        let home = Home::new(&context);
        let reader = Reader::new(&context);
        context
            .borrow_mut()
            .view_stack
            .add_named(&home, Some("home"));
        context.borrow_mut().go_home();
    }

    pub fn load_css(&self, file: &str) {
        let display = gtk::gdk::Display::default().expect("Could not get default display!");
        let provider = gtk::CssProvider::new();
        provider.load_from_file(&gtk::gio::File::for_path(file));
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    pub fn get_window(&self) -> ApplicationWindow {
        self.window.clone()
    }

    pub fn go_home(&mut self) {
        self.view_stack.set_visible_child_name("home");
        self.window.set_child(Some(&self.view_stack));
    }

    pub fn open_reader(
        context: Rc<RefCell<Self>>,
        sources: Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Files!");
        sources.iter().for_each(|f| {
            println!("{}", f);
        });

        println!("reading file...");

        let pdf_file = PDF::from_file(sources.first().unwrap())
            .expect(format!("Could not open file `{}`", sources.first().unwrap()).as_str());

        let pages = pdf_file.render(
            pdf2image::Pages::Range(1..=10),
            RenderOptionsBuilder::default().pdftocairo(true).build()?,
        )?;

        println!("File read!");
        let page_count = pdf_file.page_count();

        context.borrow_mut().reader_state = Some(ReaderState::new(
            sources.first().unwrap(),
            pages,
            page_count as usize,
        ));
        println!("ReaderState created!");

        let reader = Reader::new(&context);
        context
            .borrow_mut()
            .view_stack
            .add_named(&reader, Some("reader"));
        context
            .borrow_mut()
            .view_stack
            .set_visible_child_name("reader");

        Ok(())
    }
}
