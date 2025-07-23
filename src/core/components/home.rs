use crate::core::app::FiapoController;
use crate::core::server::{Source, SourceType};
use glib::MainContext;
use glib::clone;
use gtk::prelude::{ButtonExt, FileExt, ListModelExtManual, WidgetExt};
use gtk::{Button, CenterBox, gio, glib};
use gtk4 as gtk;
use log::{debug, warn};
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

        self._container.set_center_widget(Some(&open_button));

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
}
