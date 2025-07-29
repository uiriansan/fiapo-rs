use crate::app::FiapoController;
use crate::core::reader::{Source, SourceType};
use crate::server::{self, MangadexSearchData};
use fragile;
use glib::MainContext;
use glib::clone;
use glib::subclass::prelude::*;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, FileExt, FrameExt, ListItemExt, ListModelExtManual, WidgetExt,
};
use gtk::{Button, FlowBox, Label, ListBox, SearchEntry, gdk, gio, glib};
use gtk4::glib::object::{Cast, CastNone};
use gtk4::{self as gtk, Picture};
use log::{debug, warn};
use reqwest;
use std::cell::RefCell;
use std::ffi::OsStr;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;

#[derive(Debug, Default)]
pub struct Home {
    _controller: Rc<RefCell<FiapoController>>,
    _container: gtk::Box,
    is_searching: Arc<AtomicBool>,
}
impl Home {
    pub fn new(controller: Rc<RefCell<FiapoController>>) -> Self {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 20);
        let is_searching = Arc::new(AtomicBool::new(false));
        Self {
            _controller: controller,
            _container: container,
            is_searching,
        }
    }

    pub fn build(&self) -> gtk::Box {
        let open_button = Button::with_label("Import files");
        open_button.set_hexpand(false);
        open_button.set_cursor(gtk::gdk::Cursor::from_name("pointer", None).as_ref());

        open_button.connect_clicked(clone!(
            #[strong(rename_to = controller)]
            self._controller,
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

        let header_container = gtk::Box::new(gtk::Orientation::Horizontal, 20);
        header_container.set_vexpand(false);
        header_container.append(&manga_search_bar);
        header_container.append(&open_button);
        header_container.set_margin_top(20);
        header_container.set_margin_end(20);
        header_container.set_margin_start(20);

        self._container.append(&header_container);

        let results_list = ListBox::new();
        let scroll = gtk::ScrolledWindow::new();
        scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        // scroll.set_child(Some(&results_list));
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);

        let flow_box = FlowBox::builder()
            .row_spacing(20)
            .column_spacing(20)
            .row_spacing(10)
            .margin_top(10)
            .margin_bottom(10)
            .homogeneous(false)
            .min_children_per_line(4)
            .valign(gtk::Align::Start)
            .halign(gtk::Align::Fill)
            .selection_mode(gtk::SelectionMode::None)
            .build();
        scroll.set_child(Some(&flow_box));

        self._container.append(&scroll);

        manga_search_bar.connect_search_changed(clone!(
            #[strong(rename_to = is_searching)]
            self.is_searching,
            move |entry| {
                let search_text = entry.text().to_string();

                if is_searching.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }

                if !search_text.is_empty() {
                    results_list.remove_all();
                    flow_box.remove_all();
                    is_searching.store(true, std::sync::atomic::Ordering::Relaxed);

                    let loading_label = Label::new(Some("Searching..."));
                    loading_label.set_margin_top(50);
                    results_list.append(&loading_label);

                    glib::MainContext::default().spawn_local(clone!(
                        #[strong]
                        search_text,
                        #[strong]
                        results_list,
                        #[strong]
                        is_searching,
                        #[strong]
                        flow_box,
                        #[strong]
                        scroll,
                        async move {
                            println!("Searching for {}...", &search_text);
                            match server::search_manga(search_text).await {
                                Ok(mangas) => {
                                    results_list.remove_all();
                                    flow_box.remove_all();

                                    if mangas.is_empty() {
                                        let no_results_label = Label::new(Some("No results found"));
                                        no_results_label.set_margin_top(50);
                                        results_list.append(&no_results_label);
                                    } else {
                                        let manga_objects: Vec<MangaSearchCardDataObject> = mangas
                                            .into_iter()
                                            .map(|manga| MangaSearchCardDataObject::new(manga))
                                            .collect();

                                        let model =
                                            gio::ListStore::new::<MangaSearchCardDataObject>();
                                        for manga in manga_objects {
                                            model.append(&manga);
                                        }

                                        let factory = gtk::SignalListItemFactory::new();
                                        factory.connect_setup(move |_, list_item| {
                                            Home::create_manga_card_for_search_results(list_item)
                                        });
                                        factory.connect_bind(move |_, list_item| {
                                            Home::update_manga_card_for_search_results(list_item)
                                        });

                                        let grid_view = gtk::GridView::builder()
                                            .model(&gtk::NoSelection::new(Some(model)))
                                            .factory(&factory)
                                            .min_columns(1)
                                            .build();

                                        scroll.set_child(Some(&grid_view));
                                    }
                                }
                                Err(e) => {
                                    results_list.remove_all();
                                    flow_box.remove_all();
                                    let error_label =
                                        Label::new(Some(&format!("Search failed: {}", e)));
                                    error_label.set_margin_top(50);
                                    results_list.append(&error_label);
                                }
                            }
                            is_searching.store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                    ));
                } else {
                    results_list.remove_all();
                    flow_box.remove_all();
                }
            }
        ));

        self._container.clone()
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

    fn create_manga_card_for_search_results(list_item: &glib::Object) {
        let card_frame = gtk::AspectFrame::builder()
            .ratio(180.0 / 200.0)
            .obey_child(false)
            .width_request(200)
            .height_request(180)
            .build();
        let card = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .hexpand(false)
            .vexpand(false)
            .width_request(180)
            .height_request(200)
            .build();
        card.add_css_class("search-manga-card");
        let cover_frame = gtk::Frame::builder()
            .hexpand(false)
            .vexpand(false)
            .width_request(130)
            .height_request(185)
            .valign(gtk::Align::Start)
            .build();
        let cover = gtk::Picture::builder()
            .width_request(130)
            .height_request(185)
            .can_shrink(true)
            .hexpand(false)
            .vexpand(false)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .content_fit(gtk::ContentFit::Cover)
            .build();
        cover.add_css_class("manga-cover");
        let title_label = gtk::Label::builder()
            .label("")
            .wrap(true)
            .justify(gtk::Justification::Center)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::End)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .lines(2)
            .build();
        title_label.add_css_class("manga-title-label");

        cover_frame.set_child(Some(&cover));
        card.append(&cover_frame);
        card.append(&title_label);
        card_frame.set_child(Some(&card));

        let list_item = list_item
            .downcast_ref::<gtk::ListItem>()
            .expect("Could not downcast list_item");
        list_item.set_child(Some(&card_frame));
    }

    fn update_manga_card_for_search_results(list_item: &glib::Object) {
        let list_item = list_item
            .downcast_ref::<gtk::ListItem>()
            .expect("Could not downcast list_item");
        let manga_obj = list_item
            .item()
            .and_downcast::<MangaSearchCardDataObject>()
            .expect("Could not downcast MangaSearchCardDataObject");
        let card_frame = list_item
            .child()
            .and_downcast::<gtk::AspectFrame>()
            .expect("Could not downcast gtk::AspectFrame");
        let card = card_frame
            .child()
            .and_downcast::<gtk::Box>()
            .expect("Could not downcast gtk::Box");
        let cover_frame = card
            .first_child()
            .and_downcast::<gtk::Frame>()
            .expect("Could not downcast gtk::Frame");
        let cover = cover_frame
            .child()
            .and_downcast::<gtk::Picture>()
            .expect("Could not downcast gtk::Picture");
        let title_label = card
            .last_child()
            .and_downcast::<gtk::Label>()
            .expect("Could not downcast gtk::Label");

        let manga_data = manga_obj.data();
        title_label.set_text(if !manga_data.romaji_title.is_empty() {
            &manga_data.romaji_title
        } else {
            &manga_data.english_title
        });

        let cover_url = manga_data.cover_url;
        let cover_clone = fragile::Fragile::new(cover.clone());
        thread::spawn(move || {
            let texture_result = Home::texture_from_url(String::from(&cover_url));

            if let Ok(texture) = texture_result {
                glib::MainContext::default().invoke(move || {
                    cover_clone.get().set_paintable(Some(&texture));
                });
            } else {
                eprintln!("Failed to load texture from {}", cover_url);
            }
        });
    }

    fn texture_from_url(url: String) -> Result<gdk::Texture, Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("github.uiriansan.fiapo")
            .timeout(std::time::Duration::from_secs(5))
            .build()?;
        let result = client.get(url).send()?;
        if !result.status().is_success() {
            return Err(format!("Image request failed: {}", result.status()).into());
        }
        let img_data = result.bytes()?;

        let img_stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(&img_data));
        let pixbuf = Pixbuf::from_stream_at_scale(
            &img_stream,
            130,
            185,
            true,
            Some(&gio::Cancellable::new()),
        )?;

        Ok(gdk::Texture::for_pixbuf(&pixbuf))
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MangaSearchCardDataObject {
        pub data: RefCell<Option<MangadexSearchData>>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for MangaSearchCardDataObject {
        const NAME: &'static str = "SomeStructObject";
        type Type = super::MangaSearchCardDataObject;
    }

    impl ObjectImpl for MangaSearchCardDataObject {}
}

glib::wrapper! {
    pub struct MangaSearchCardDataObject(ObjectSubclass<imp::MangaSearchCardDataObject>);
}
impl MangaSearchCardDataObject {
    pub fn new(data: MangadexSearchData) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.imp().data.replace(Some(data));
        obj
    }
    pub fn data(&self) -> MangadexSearchData {
        self.imp().data.borrow().as_ref().unwrap().clone()
    }
}
