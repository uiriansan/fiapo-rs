use crate::app::FiapoController;
use crate::core::reader::{Source, SourceType};
use crate::server;
use crate::ui::components::card::{Card, MangadexSearchDataObject};
use glib::MainContext;
use glib::clone;
use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, FileExt, ListItemExt, ListModelExtManual, WidgetExt,
};
use gtk::{Button, Label, SearchEntry, gio, glib};
use gtk4 as gtk;
use gtk4::glib::object::{Cast, CastNone};
use log::{debug, warn};
use std::cell::RefCell;
use std::ffi::OsStr;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[derive(Debug, Default)]
pub struct Home {
    controller: Rc<RefCell<FiapoController>>,
    container: gtk::Box,
    is_searching: Arc<AtomicBool>,
}
impl Home {
    pub fn new(controller: Rc<RefCell<FiapoController>>) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 20);
        let is_searching = Arc::new(AtomicBool::new(false));

        Self {
            controller: controller,
            container: container,
            is_searching,
        }
    }

    pub fn build(&self) -> gtk::Box {
        let open_button = Button::with_label("Import files");
        open_button.set_hexpand(false);
        open_button.set_cursor(gtk::gdk::Cursor::from_name("pointer", None).as_ref());

        open_button.connect_clicked(clone!(
            #[strong(rename_to = controller)]
            self.controller,
            move |_| {
                MainContext::default().spawn_local({
                    let controller = Rc::clone(&controller);
                    async move {
                        let window = {
                            let ctrl = controller.borrow();
                            ctrl.get_window()
                        };

                        let mut total_page_count: usize = 0;

                        match Home::open_file_dialog(&window).await {
                            Ok(files) => {
                                let mut files_vec: Vec<Source> = Vec::new();

                                for (i, file) in files.iter::<gio::File>().enumerate() {
                                    let file = file.unwrap();

                                    let path = file.path().unwrap();
                                    let source_type: SourceType = match path.is_file() {
                                        true => match path.extension().and_then(OsStr::to_str) {
                                            Some("pdf") => SourceType::Pdf,
                                            Some("png") => SourceType::ImageSequence,
                                            Some("jpg") => SourceType::ImageSequence,
                                            _ => SourceType::Pdf,
                                        },
                                        _ => SourceType::Directory,
                                    };

                                    let str_path = path.clone().to_str().unwrap().to_string();

                                    // Ignore directories for now
                                    if source_type == SourceType::Directory {
                                        warn!("Skipping directory: {}", str_path);
                                        continue;
                                    }

                                    let should_keep_pdf_object: bool = i == 0;
                                    let source =
                                        Source::new(source_type, path, should_keep_pdf_object);
                                    let page_count = source.get_page_count();

                                    // Ignore empty PDFs
                                    if page_count <= 0 {
                                        warn!("Skipping empty file: {}", str_path);
                                        continue;
                                    }

                                    total_page_count += page_count;
                                    files_vec.push(source);
                                }

                                files_vec.sort();

                                controller
                                    .borrow_mut()
                                    .server
                                    .set_sources(files_vec, total_page_count);
                                debug!("{:?}", controller.borrow());
                                FiapoController::open_reader(controller);
                            }
                            Err(e) => warn!("Could not open file: {}", e),
                        }
                    }
                });
            }
        ));
        let manga_search_bar = SearchEntry::new();
        manga_search_bar.set_search_delay(500); // ms
        manga_search_bar.set_hexpand(true);
        manga_search_bar.set_placeholder_text(Some("Search for mangas..."));

        let headercontainer = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        headercontainer.set_vexpand(false);
        headercontainer.append(&manga_search_bar);
        headercontainer.append(&open_button);
        headercontainer.set_margin_top(10);
        headercontainer.set_margin_end(10);
        headercontainer.set_margin_start(10);

        self.container.append(&headercontainer);

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);

        self.container.append(&scroll);

        manga_search_bar.connect_search_changed(clone!(
            #[strong(rename_to = is_searching)]
            self.is_searching,
            move |entry| {
                let search_text = entry.text().to_string();

                if is_searching.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }

                if !search_text.is_empty() {
                    is_searching.store(true, std::sync::atomic::Ordering::Relaxed);

                    let loading_label = Label::new(Some("Searching..."));
                    scroll.set_child(Some(&loading_label));

                    glib::MainContext::default().spawn_local(clone!(
                        #[strong]
                        search_text,
                        #[strong]
                        is_searching,
                        #[strong]
                        scroll,
                        async move {
                            println!("Searching for {}...", &search_text);
                            match server::search_manga(search_text).await {
                                Ok(mangas) => {
                                    if mangas.is_empty() {
                                        let no_results_label = Label::new(Some("No results found"));
                                        scroll.set_child(Some(&no_results_label));
                                    } else {
                                        let manga_objects: Vec<MangadexSearchDataObject> = mangas
                                            .into_iter()
                                            .map(|manga| MangadexSearchDataObject::new(manga))
                                            .collect();

                                        let model =
                                            gio::ListStore::new::<MangadexSearchDataObject>();
                                        for manga in manga_objects {
                                            model.append(&manga);
                                        }

                                        let factory = gtk::SignalListItemFactory::new();
                                        factory.connect_setup(move |_, list_item| {
                                            Home::create_cards_for_grid_view(list_item);
                                        });
                                        factory.connect_bind(move |_, list_item| {
                                            Home::update_grid_view_cards(list_item);
                                        });

                                        let grid_view = gtk::GridView::builder()
                                            .model(&gtk::NoSelection::new(Some(model)))
                                            .factory(&factory)
                                            .min_columns(3)
                                            .build();
                                        grid_view.add_css_class("search-grid-view");

                                        scroll.set_child(Some(&grid_view));
                                    }
                                }
                                Err(e) => {
                                    let error_label =
                                        Label::new(Some(&format!("Search failed: {}", e)));
                                    scroll.set_child(Some(&error_label));
                                }
                            }
                            is_searching.store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                    ));
                }
            }
        ));

        self.container.clone()
    }

    async fn open_file_dialog(
        window: &gtk::ApplicationWindow,
    ) -> Result<gio::ListModel, glib::Error> {
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        let file_filter = gtk::FileFilter::new();
        file_filter.set_name(Some("Images and PDF files"));
        file_filter.add_mime_type("image/*");
        file_filter.add_mime_type("application/pdf");
        filters.append(&file_filter);

        let file_dialog = gtk::FileDialog::builder()
            .title("Select one or more files")
            .accept_label("Import")
            .modal(true)
            .filters(&filters)
            .build();

        file_dialog.open_multiple_future(Some(window)).await
    }

    fn create_cards_for_grid_view(list_item: &glib::Object) {
        let list_item = list_item
            .downcast_ref::<gtk::ListItem>()
            .expect("Could not downgcast ListItem");
        let card = Card::new();
        list_item.set_child(Some(&card));
    }

    fn update_grid_view_cards(list_item: &glib::Object) {
        let list_item = list_item
            .downcast_ref::<gtk::ListItem>()
            .expect("Could not downcast ListItem");
        let card_object = list_item
            .item()
            .and_downcast::<MangadexSearchDataObject>()
            .expect("Could not downcast MangadexSearchDataObject");
        let card_data = card_object.data();
        let card = list_item
            .child()
            .and_downcast::<Card>()
            .expect("Could not downcast Card");
        card.update(card_data);
    }
}
