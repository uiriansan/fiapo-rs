use env_logger::Env;
use gtk::prelude::*;
use gtk::{Application, glib};
use gtk4 as gtk;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

mod app;
mod core;
mod server;
mod ui;
use app::FiapoController;

use server::get_r_manga;

const APP_ID: &str = "github.uiriansan.fiapo";
const CONFIG_FILE: &str = "~/.config/fiapo/fiapo.toml";
const CSS_FILE: &str = "../resources/styles/main.css";

#[tokio::main]
async fn main() -> glib::ExitCode {
    init_logger();

    let application = Application::builder().application_id(APP_ID).build();

    application.connect_activate(|app| {
        let controller = Rc::new(RefCell::new(FiapoController::new(app)));
        controller.borrow_mut().load_config(CONFIG_FILE);
        controller.borrow().load_css(CSS_FILE);
        FiapoController::build_ui(controller);
    });
    application.run()
}

fn init_logger() {
    // TODO: Save to a file. Looks like 'env_logger' can't do it.
    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            let warn_style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "{warn_style}[{}]:{warn_style:#} {}",
                record.level(),
                record.args()
            )
        })
        .init();
}
