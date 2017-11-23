/*
 * TODO: show the gtk scrollbars instead of the Servo scrollbars?
 * TODO: send CloseBrowser event.
 */

extern crate epoxy;
extern crate gdk;
extern crate gdk_sys;
extern crate glib_itc;
extern crate gtk;
extern crate servo;
extern crate shared_library;

mod convert;
mod eventloop;
pub mod view;
mod window;

pub use view::WebView;
