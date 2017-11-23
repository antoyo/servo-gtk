use gdk::{
    ModifierType,
    keyval_to_unicode,
    CONTROL_MASK,
    META_MASK,
    SHIFT_MASK,
    SUPER_MASK,
};
use servo::msg::constellation_msg::{ALT, CONTROL, SHIFT, SUPER};
use gdk::enums::key as gdk_key;
use gdk_sys::{GDK_BUTTON_MIDDLE, GDK_BUTTON_PRIMARY, GDK_BUTTON_SECONDARY};
use servo::msg::constellation_msg::{Key, KeyModifiers};
use servo::script_traits::MouseButton;

pub fn modifiers(modifiers: ModifierType) -> KeyModifiers {
    let mut result = KeyModifiers::empty();
    if modifiers.contains(META_MASK) {
        result.insert(ALT);
    }
    if modifiers.contains(SUPER_MASK) {
        result.insert(SUPER);
    }
    if modifiers.contains(CONTROL_MASK) {
        result.insert(CONTROL);
    }
    if modifiers.contains(SHIFT_MASK) {
        result.insert(SHIFT);
    }
    result
}

pub fn mouse_button(gtk_button: u32) -> MouseButton {
    match gtk_button as i32 {
        GDK_BUTTON_MIDDLE => MouseButton::Middle,
        GDK_BUTTON_PRIMARY => MouseButton::Left,
        GDK_BUTTON_SECONDARY => MouseButton::Right,
        _ => panic!("unexpected value for gtk_button"),
    }
}

pub fn key(gdk_key: gdk_key::Key) -> (Option<char>, Option<Key>) {
    let unicode =
        keyval_to_unicode(gdk_key)
            .and_then(|char|
                      if char.is_control() {
                          None
                      }
                      else {
                          Some(char)
                      }
            );
    let key =
        match gdk_key {
            gdk_key::space => Key::Space,
            gdk_key::apostrophe => Key::Apostrophe,
            gdk_key::comma => Key::Comma,
            gdk_key::minus => Key::Minus,
            gdk_key::period => Key::Period,
            gdk_key::slash => Key::Slash,
            gdk_key::_0 => Key::Num0,
            gdk_key::_1 => Key::Num1,
            gdk_key::_2 => Key::Num2,
            gdk_key::_3 => Key::Num3,
            gdk_key::_4 => Key::Num4,
            gdk_key::_5 => Key::Num5,
            gdk_key::_6 => Key::Num6,
            gdk_key::_7 => Key::Num7,
            gdk_key::_8 => Key::Num8,
            gdk_key::_9 => Key::Num9,
            gdk_key::semicolon => Key::Semicolon,
            gdk_key::equal => Key::Equal,
            gdk_key::A | gdk_key::a => Key::A,
            gdk_key::B | gdk_key::b => Key::B,
            gdk_key::C | gdk_key::c => Key::C,
            gdk_key::D | gdk_key::d => Key::D,
            gdk_key::E | gdk_key::e => Key::E,
            gdk_key::F | gdk_key::f => Key::F,
            gdk_key::G | gdk_key::g => Key::G,
            gdk_key::H | gdk_key::h => Key::H,
            gdk_key::I | gdk_key::i => Key::I,
            gdk_key::J | gdk_key::j => Key::J,
            gdk_key::K | gdk_key::k => Key::K,
            gdk_key::L | gdk_key::l => Key::L,
            gdk_key::M | gdk_key::m => Key::M,
            gdk_key::N | gdk_key::n => Key::N,
            gdk_key::O | gdk_key::o => Key::O,
            gdk_key::P | gdk_key::p => Key::P,
            gdk_key::Q | gdk_key::q => Key::Q,
            gdk_key::R | gdk_key::r => Key::R,
            gdk_key::S | gdk_key::s => Key::S,
            gdk_key::T | gdk_key::t => Key::T,
            gdk_key::U | gdk_key::u => Key::U,
            gdk_key::V | gdk_key::v => Key::V,
            gdk_key::W | gdk_key::w => Key::W,
            gdk_key::X | gdk_key::x => Key::X,
            gdk_key::Y | gdk_key::y => Key::Y,
            gdk_key::Z | gdk_key::z => Key::Z,
            gdk_key::bracketleft => Key::LeftBracket,
            gdk_key::backslash => Key::Backslash,
            gdk_key::bracketright => Key::RightBracket,
            gdk_key::dead_grave => Key::GraveAccent,
            //gdk_key:: => Key::World1, // TODO
            //gdk_key:: => Key::World2,
            gdk_key::Escape => Key::Escape,
            gdk_key::Return => Key::Enter,
            gdk_key::Tab => Key::Tab,
            gdk_key::BackSpace => Key::Backspace,
            gdk_key::Insert => Key::Insert,
            gdk_key::Delete => Key::Delete,
            gdk_key::Right => Key::Right,
            gdk_key::Left => Key::Left,
            gdk_key::Down => Key::Down,
            gdk_key::Up => Key::Up,
            gdk_key::Page_Up => Key::PageUp,
            gdk_key::Page_Down => Key::PageDown,
            gdk_key::Home => Key::Home,
            gdk_key::End => Key::End,
            gdk_key::Caps_Lock => Key::CapsLock,
            gdk_key::Scroll_Lock => Key::ScrollLock,
            gdk_key::Num_Lock => Key::NumLock,
            gdk_key::_3270_PrintScreen => Key::PrintScreen, // TODO: tes)t
            gdk_key::Pause => Key::Pause,
            gdk_key::F1 => Key::F1,
            gdk_key::F2 => Key::F2,
            gdk_key::F3 => Key::F3,
            gdk_key::F4 => Key::F4,
            gdk_key::F5 => Key::F5,
            gdk_key::F6 => Key::F6,
            gdk_key::F7 => Key::F7,
            gdk_key::F8 => Key::F8,
            gdk_key::F9 => Key::F9,
            gdk_key::F10 => Key::F10,
            gdk_key::F11 => Key::F11,
            gdk_key::F12 => Key::F12,
            gdk_key::F13 => Key::F13,
            gdk_key::F14 => Key::F14,
            gdk_key::F15 => Key::F15,
            gdk_key::F16 => Key::F16,
            gdk_key::F17 => Key::F17,
            gdk_key::F18 => Key::F18,
            gdk_key::F19 => Key::F19,
            gdk_key::F20 => Key::F20,
            gdk_key::F21 => Key::F21,
            gdk_key::F22 => Key::F22,
            gdk_key::F23 => Key::F23,
            gdk_key::F24 => Key::F24,
            gdk_key::F25 => Key::F25,
            gdk_key::KP_0 => Key::Kp0,
            gdk_key::KP_1 => Key::Kp1,
            gdk_key::KP_2 => Key::Kp2,
            gdk_key::KP_3 => Key::Kp3,
            gdk_key::KP_4 => Key::Kp4,
            gdk_key::KP_5 => Key::Kp5,
            gdk_key::KP_6 => Key::Kp6,
            gdk_key::KP_7 => Key::Kp7,
            gdk_key::KP_8 => Key::Kp8,
            gdk_key::KP_9 => Key::Kp9,
            gdk_key::KP_Decimal => Key::KpDecimal,
            gdk_key::KP_Divide => Key::KpDivide,
            gdk_key::KP_Multiply => Key::KpMultiply,
            gdk_key::KP_Subtract => Key::KpSubtract,
            gdk_key::KP_Add => Key::KpAdd,
            gdk_key::KP_Enter => Key::KpEnter,
            gdk_key::KP_Equal => Key::KpEqual,
            gdk_key::Shift_L => Key::LeftShift,
            gdk_key::Control_L => Key::LeftControl,
            gdk_key::Alt_L => Key::LeftAlt,
            gdk_key::Super_L => Key::LeftSuper,
            gdk_key::Shift_R => Key::RightShift,
            gdk_key::Control_R => Key::RightControl,
            gdk_key::Alt_R => Key::RightAlt,
            gdk_key::Super_R => Key::RightSuper,
            gdk_key::Menu => Key::Menu,
            //gdk_key:: => Key::NavigateBackward, // TODO
            //gdk_key:: => Key::NavigateForward,
            _ => return
                if let Some(key) = unicode {
                    (Some(key), Some(Key::A)) // FIXME: don't send A.
                }
                else {
                    (None, None)
                }
        };
    (unicode, Some(key))
}
