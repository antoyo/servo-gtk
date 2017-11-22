use gdk_sys::{GDK_BUTTON_MIDDLE, GDK_BUTTON_PRIMARY, GDK_BUTTON_SECONDARY};
use servo::script_traits::MouseButton;

pub fn mouse_button(gtk_button: u32) -> MouseButton {
    match gtk_button as i32 {
        GDK_BUTTON_MIDDLE => MouseButton::Middle,
        GDK_BUTTON_PRIMARY => MouseButton::Left,
        GDK_BUTTON_SECONDARY => MouseButton::Right,
        _ => panic!("unexpected value for gtk_button"),
    }
}
