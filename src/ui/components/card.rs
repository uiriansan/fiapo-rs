use fragile;
use glib::Object;
use glib::subclass::prelude::*;
use gtk::prelude::*;
use gtk::subclass::{box_::BoxImpl, widget::WidgetImpl};
use gtk::{gdk, gdk_pixbuf, gio, glib};
use gtk4 as gtk;
use std::cell::{OnceCell, RefCell};
use std::rc::Rc;
use std::thread;

use crate::server::MangadexSearchData;

const CARD_COVER_WIDTH: i32 = 130;
const CARD_COVER_HEIGHT: i32 = 170;

mod card_imp {
    use super::*;

    #[derive(Default)]
    pub struct CardExt {
        pub cover_picture: OnceCell<gtk::Picture>,
        pub title_label: OnceCell<gtk::Label>,
        pub author_label: OnceCell<gtk::Label>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for CardExt {
        const NAME: &'static str = "FiapoMangaCard";
        type Type = super::Card;
        type ParentType = gtk::Box;
    }
    impl WidgetImpl for CardExt {}
    impl BoxImpl for CardExt {}
    impl ObjectImpl for CardExt {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_card();
        }
    }
    impl CardExt {
        fn setup_card(&self) {
            let card = self.obj();

            card.set_orientation(gtk::Orientation::Vertical);
            card.set_spacing(10);
            card.set_hexpand(false);
            card.set_vexpand(false);
            card.add_css_class("manga-card");
            card.set_cursor(gtk::gdk::Cursor::from_name("pointer", None).as_ref());

            let cover_picture = gtk::Picture::builder()
                .width_request(CARD_COVER_WIDTH)
                .height_request(CARD_COVER_HEIGHT)
                .can_shrink(true)
                .hexpand(false)
                .vexpand(false)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Start)
                .content_fit(gtk::ContentFit::Cover)
                .build();
            cover_picture.add_css_class("manga-card-cover");

            let cover_box = gtk::Box::builder()
                .width_request(CARD_COVER_WIDTH)
                .height_request(CARD_COVER_HEIGHT)
                .hexpand(false)
                .vexpand(false)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Start)
                .build();
            cover_box.add_css_class("manga-card-cover-box");
            cover_box.append(&cover_picture);

            let title_label = gtk::Label::builder()
                .label("")
                .wrap(true)
                .justify(gtk::Justification::Center)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::End)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .lines(2)
                .max_width_chars(25)
                .build();
            title_label.add_css_class("manga-card-title");

            let author_label = gtk::Label::builder()
                .label("")
                .wrap(false)
                .justify(gtk::Justification::Center)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::End)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .lines(1)
                .max_width_chars(25)
                .build();
            author_label.add_css_class("manga-card-author");

            card.append(&cover_box);
            card.append(&title_label);
            card.append(&author_label);

            self.cover_picture
                .set(cover_picture)
                .expect("'cover_picture' is already set");
            self.title_label
                .set(title_label)
                .expect("'title_label' is already set");
            self.author_label
                .set(author_label)
                .expect("'author_label' is already set");
        }
    }
}
glib::wrapper! {
    pub struct Card(ObjectSubclass<card_imp::CardExt>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}
impl Card {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn get_cover_picture(&self) -> Option<&gtk::Picture> {
        self.imp().cover_picture.get()
    }
    pub fn get_title_label(&self) -> Option<&gtk::Label> {
        self.imp().title_label.get()
    }
    pub fn get_author_label(&self) -> Option<&gtk::Label> {
        self.imp().author_label.get()
    }

    pub fn update(&self, manga_data: MangadexSearchData) {
        if let Some(cover_picture) = self.get_cover_picture() {
            let cover_url = manga_data.cover_url;
            let cover_picture_clone = fragile::Sticky::new(cover_picture.clone());

            thread::spawn(move || {
                let texture_result = Card::texture_from_url(String::from(&cover_url));

                match texture_result {
                    Ok(texture) => glib::MainContext::default().invoke(move || {
                        fragile::stack_token!(tok);
                        cover_picture_clone.get(tok).set_paintable(Some(&texture));
                    }),
                    Err(e) => {
                        eprintln!("Failed to load texture from {}: {}", cover_url, e);
                        glib::MainContext::default().invoke(move || {
                            fragile::stack_token!(tok);
                            cover_picture_clone.get(tok).set_visible(false);
                        });
                    }
                }
            });
        }
        let manga_title = if !manga_data.romaji_title.is_empty() {
            manga_data.romaji_title.clone()
        } else {
            manga_data.english_title.clone()
        };
        if let Some(title_label) = self.get_title_label() {
            title_label.set_text(&manga_title);
        }
        let manga_authors = format!(
            "{}{}",
            &manga_data.author,
            if manga_data.artist == manga_data.author {
                "".to_string()
            } else {
                format!(", {}", &manga_data.artist)
            }
        );
        if let Some(author_label) = self.get_author_label() {
            author_label.set_text(&manga_authors);
        }

        self.set_tooltip_text(Some(&manga_title));

        let click_controller = gtk::GestureClick::builder()
            .button(0) // All buttons
            .build();

        let click_timeout_id: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));

        click_controller.connect_pressed(move |gesture, press_count, _x, _y| {
            let button = gesture.current_button();

            if let Some(timeout_id) = click_timeout_id.borrow_mut().take() {
                timeout_id.remove();
            }
            match (button, press_count) {
                (1, 1) => {
                    // Left button, single click
                    let timeout_id = click_timeout_id.clone();
                    let manga_title = manga_title.clone();
                    let manga_authors = manga_authors.clone();

                    let timeout =
                        glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
                            println!(
                                "{:0>3}:\n    {}\n    {}\n",
                                manga_data.id.to_string(),
                                manga_title,
                                manga_authors
                            );

                            *timeout_id.borrow_mut() = None;
                            glib::ControlFlow::Break
                        });
                    *click_timeout_id.borrow_mut() = Some(timeout);
                }
                (1, 2) => {
                    // Left button. double click
                    println!("Open reader");
                }
                (3, 1) => {
                    println!("Context menu");
                    // Right button, single click
                }
                _ => (),
            }
        });
        self.add_controller(click_controller);
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
        let pixbuf = gdk_pixbuf::Pixbuf::from_stream_at_scale(
            &img_stream,
            CARD_COVER_WIDTH,
            CARD_COVER_HEIGHT,
            true,
            Some(&gio::Cancellable::new()),
        )?;

        Ok(gdk::Texture::for_pixbuf(&pixbuf))
    }
}

mod car_data_imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MangadexSearchDataObject {
        pub data: RefCell<Option<MangadexSearchData>>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for MangadexSearchDataObject {
        const NAME: &'static str = "FiapoMangaSearchDataObject";
        type Type = super::MangadexSearchDataObject;
    }

    impl ObjectImpl for MangadexSearchDataObject {}
}
glib::wrapper! {
    /// glib::Object wrapper around MangadexSearchData, so we can append the data to the GridView's model.
    pub struct MangadexSearchDataObject(ObjectSubclass<car_data_imp::MangadexSearchDataObject>);
}
impl MangadexSearchDataObject {
    pub fn new(data: MangadexSearchData) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.imp().data.replace(Some(data));
        obj
    }
    pub fn data(&self) -> MangadexSearchData {
        self.imp().data.borrow().as_ref().unwrap().clone()
    }
}
