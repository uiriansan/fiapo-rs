use crate::app::FiapoController;
use crate::core::reader::{Source, SourceType};
use crate::server::{self, MangadexSearchData};
use glib::MainContext;
use glib::clone;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, FileExt, ListModelExtManual, OrientableExt, WidgetExt,
};
use gtk::{Button, CenterBox, Label, ListBox, SearchEntry, gdk, gio, glib};
use gtk4 as gtk;
use log::{debug, warn};
use reqwest;
use std::cell::RefCell;
use std::ffi::OsStr;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Home {
    _controller: Rc<RefCell<FiapoController>>,
    _container: CenterBox,
}
impl Home {
    pub fn new(controller: Rc<RefCell<FiapoController>>) -> Self {
        let container = CenterBox::new();
        Self {
            _controller: controller,
            _container: container,
        }
    }

    pub fn build(&self) -> CenterBox {
        let open_button = Button::with_label("Import files");
        open_button.set_hexpand(true);
        open_button.set_vexpand(true);
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

        let results_list = ListBox::new();
        let scroll = gtk::ScrolledWindow::new();
        scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        scroll.set_child(Some(&results_list));
        scroll.set_vexpand(true);
        self._container.set_center_widget(Some(&scroll));

        let manga_search_bar = SearchEntry::new();
        manga_search_bar.set_search_delay(500); // ms
        manga_search_bar.set_placeholder_text(Some("Search for mangas..."));
        manga_search_bar.connect_search_changed(move |entry| {
            MainContext::default().spawn_local(clone!(
                #[strong]
                entry,
                #[strong]
                results_list,
                async move {
                    if !entry.text().is_empty() {
                        println!("Searching...");
                        if let Ok(mangas) = server::search_manga(entry.text().as_str()).await {
                            results_list.remove_all();
                            mangas.iter().for_each(|manga: &MangadexSearchData| {
                                let manga_container = CenterBox::new();
                                manga_container.set_orientation(gtk::Orientation::Horizontal);
                                let cover = gtk::Picture::new();
                                cover.set_size_request(100, 100);
                                results_list.append(&manga_container);

                                let cover_url = &manga.cover_url;
                                glib::spawn_future_local(clone!(
                                    #[strong]
                                    cover_url,
                                    #[strong]
                                    cover,
                                    async move {
                                        if let Ok(texture) =
                                            Home::texture_from_url(String::from(&cover_url)).await
                                        {
                                            glib::idle_add_local_once(clone!(
                                                #[strong]
                                                cover,
                                                #[strong]
                                                texture,
                                                move || {
                                                    cover.set_paintable(Some(&texture));
                                                }
                                            ));
                                        } else {
                                            eprintln!("Failed to load texture from {}", cover_url);
                                        }
                                    }
                                ));

                                manga_container.set_start_widget(Some(&cover));
                                let manga_body_container =
                                    gtk::Box::new(gtk::Orientation::Vertical, 10);

                                let eng_title_label =
                                    Label::new(Some(&manga.english_title.as_str()));
                                let romaji_title_label =
                                    Label::new(Some(&manga.romaji_title.as_str()));
                                let author_label = Label::new(Some(
                                    format!(
                                        "{}{}",
                                        &manga.author,
                                        if &manga.artist != &manga.author {
                                            format!(",{}", &manga.artist)
                                        } else {
                                            "".to_string()
                                        }
                                    )
                                    .as_str(),
                                ));
                                manga_body_container.append(&eng_title_label);
                                manga_body_container.append(&romaji_title_label);
                                manga_body_container.append(&author_label);
                                manga_container.set_center_widget(Some(&manga_body_container));
                            });
                        }
                    }
                }
            ));
        });

        let header_container = CenterBox::new();
        header_container.set_orientation(gtk::Orientation::Horizontal);
        header_container.set_start_widget(Some(&manga_search_bar));
        header_container.set_end_widget(Some(&open_button));
        header_container.set_margin_top(20);
        header_container.set_margin_end(50);
        header_container.set_margin_start(50);

        self._container.set_orientation(gtk::Orientation::Vertical);
        self._container.set_start_widget(Some(&header_container));

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

    async fn texture_from_url(url: String) -> Result<gdk::Texture, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .user_agent("github.uiriansan.fiapo")
            .timeout(std::time::Duration::from_secs(5))
            .build()?;
        let result = client.get(url).send().await?;
        if !result.status().is_success() {
            return Err(format!("Image request failed: {}", result.status()).into());
        }
        let img_data = result.bytes().await?;

        let img_stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(&img_data));
        let pixbuf = Pixbuf::from_stream_future(&img_stream).await?;
        Ok(gdk::Texture::for_pixbuf(&pixbuf))
    }
}
