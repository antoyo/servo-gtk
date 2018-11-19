extern crate gobject_gen;
extern crate glib_sys as glib_ffi;
extern crate gobject_sys as gobject_ffi;
extern crate gtk;
extern crate gtk_sys as gtk_ffi;
extern crate servo_gtk;

use servo_gtk::WebView;

use gobject_gen::gobject_gen;
use gtk::{
    ContainerExt,
    GtkWindowExt,
    Inhibit,
    Orientation,
    WidgetExt,
    Window,
    WindowType,
};

#[macro_use]
extern crate glib;

gobject_gen! {
    #![gir_file="Gtk-3.0"]

    #[generate("ServoGtk.gir")]
    class ServoView : gtk::GLArea {
    }

    impl ServoView {
    }
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let servo_view = ServoView::new();
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Servo GTK+");
    window.set_default_size(350, 70);
    let vbox = gtk::Box::new(Orientation::Vertical, 0);
    window.add(&vbox);
    vbox.add(&servo_view);
    window.show_all();

    let servo_webview = servo_view.clone();

    servo_view.connect_realize(move |_| {
        WebView::new(servo_webview.clone());
    });

    servo_view.set_hexpand(true);
    servo_view.set_vexpand(true);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}
