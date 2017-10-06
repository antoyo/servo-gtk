extern crate gdk;
extern crate gtk;
extern crate servo_gtk;

use std::cell::RefCell;
use std::rc::Rc;

use gdk::enums::key;
use gtk::{
    ContainerExt,
    Inhibit,
    WidgetExt,
    Window,
    WindowType,
};
use gtk::Orientation::Vertical;
use servo_gtk::WebView;

fn main() {
    gtk::init().unwrap();

    let gtk_window = Window::new(WindowType::Toplevel);
    gtk_window.set_size_request(800, 600);

    let vbox = gtk::Box::new(Vertical, 0);
    gtk_window.add(&vbox);

    let webview = WebView::new();
    vbox.add(&webview.view());

    gtk_window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk_window.show_all();

    {
        let webview = webview.clone();
        gtk_window.connect_key_press_event(move |_, event| {
            if event.get_keyval() == key::R {
                webview.reload();
            }
            Inhibit(false)
        });
    }

    gtk::main();
}
