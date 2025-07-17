// use crate::core::components::icon_button;
// use gtk::Box;
// use gtk::prelude::*;
// use gtk4 as gtk;

// pub fn create_home() -> Box {
//     let container = Box::new(gtk::Orientation::Vertical, 0);
//     container.set_hexpand(true);
//     container.set_vexpand(true);

//     let btn_open = icon_button::create("Open");
//     btn_open.set_cursor(gtk4::gdk::Cursor::from_name("pointer", None).as_ref());
//     {
//         let window = Rc::clone(&window);
//         btn_open.connect_clicked(move |_| {
//             let window = Rc::clone(&window);
//             MainContext::default().spawn_local(async move {
//                 open_file(&window).await;
//             });
//         });
//     }
//     container.append(&btn_open);

//     return container;
// }
