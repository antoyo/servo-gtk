/*
 * TODO: show the gtk scrollbars instead of the Servo scrollbars?
 * TODO: key (WindowEvent::KeyEvent) and button events.
 * TODO: send CloseBrowser event.
 */

extern crate epoxy;
extern crate gdk;
extern crate glib_itc;
extern crate gtk;
extern crate servo;
extern crate shared_library;

mod eventloop;
pub mod view;
mod window;

pub use view::WebView;
