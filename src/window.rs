use std::cell::RefCell;
use std::rc::Rc;

use gdk;
use gdk::{Display, WindowExt};
use gtk::{GLAreaExt, WidgetExt};
use servo::BrowserId;
use servo::compositing::compositor_thread::EventLoopWaker;
use servo::compositing::windowing::WindowMethods;
use servo::euclid::{Point2D, ScaleFactor, Size2D, TypedPoint2D, TypedRect, TypedSize2D};
use servo::gl;
use servo::ipc_channel::ipc;
use servo::msg::constellation_msg::{Key, KeyModifiers};
use servo::net_traits::net_error_list::NetError;
use servo::script_traits::LoadData;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::servo_url::ServoUrl;
use servo::style_traits::cursor::Cursor;
use servo::style_traits::DevicePixel;

use view::View;

struct Allocation {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

pub struct GtkWindow {
    can_go_back: RefCell<bool>,
    can_go_forward: RefCell<bool>,
    gl: Rc<gl::Gl>,
    title: RefCell<Option<String>>,
    title_callback: RefCell<Option<Box<Fn(Option<String>)>>>,
    url: RefCell<Option<String>>,
    url_callback: RefCell<Option<Box<Fn(String)>>>,
    view: View,
    waker: Box<EventLoopWaker>,
}

impl GtkWindow {
    pub fn new(gl: Rc<gl::Gl>, view: View, waker: Box<EventLoopWaker>) -> Self {
        GtkWindow {
            can_go_back: RefCell::new(false),
            can_go_forward: RefCell::new(false),
            gl,
            title: RefCell::new(None),
            title_callback: RefCell::new(None),
            url: RefCell::new(None),
            url_callback: RefCell::new(None),
            view,
            waker,
        }
    }

    pub fn can_go_back(&self) -> bool {
        *self.can_go_back.borrow()
    }

    pub fn can_go_forward(&self) -> bool {
        *self.can_go_forward.borrow()
    }

    pub fn connect_title_changed<F: Fn(Option<String>) + 'static>(&self, callback: F) {
        *self.title_callback.borrow_mut() = Some(Box::new(callback));
    }

    pub fn connect_url_changed<F: Fn(String) + 'static>(&self, callback: F) {
        *self.url_callback.borrow_mut() = Some(Box::new(callback));
    }

    fn get_geometry(&self) -> Allocation {
        let allocation = self.view.get_allocation();
        let (mut width, mut height) = (allocation.width as u32, allocation.height as u32);

        #[cfg(target_os = "windows")]
        let factor = super::utils::windows_hidpi_factor();
        #[cfg(not(target_os = "windows"))]
        let factor = 1.0f32;

        width /= factor as u32;
        height /= factor as u32;

        let x = allocation.x as u32;
        let y = allocation.y as u32;

        Allocation {
            x,
            y,
            width,
            height,
        }
    }

    pub fn get_title(&self) -> Option<String> {
        self.title.borrow().clone()
    }

    pub fn get_url(&self) -> Option<String> {
        self.url.borrow().clone()
    }
}

impl WindowMethods for GtkWindow {
    fn prepare_for_composite(&self, _width: usize, _height: usize) -> bool {
        self.view.make_current();
        true
    }

    fn present(&self) {
        self.view.queue_render();
    }

    fn supports_clipboard(&self) -> bool {
        false
    }

    fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        self.waker.clone()
    }

    fn gl(&self) -> Rc<gl::Gl> {
        self.gl.clone()
    }

    fn hidpi_factor(&self) -> ScaleFactor<f32, DeviceIndependentPixel, DevicePixel> {
        ScaleFactor::new(self.view.get_scale_factor() as f32)
    }

    fn framebuffer_size(&self) -> TypedSize2D<u32, DevicePixel> {
        let geometry = self.get_geometry();
        let scale_factor = self.view.get_scale_factor() as u32;
        TypedSize2D::new(scale_factor * geometry.width as u32, scale_factor * geometry.height as u32)
    }

    fn window_rect(&self) -> TypedRect<u32, DevicePixel> {
        TypedRect::new(TypedPoint2D::new(0, 0), self.framebuffer_size())
    }

    fn size(&self) -> TypedSize2D<f32, DeviceIndependentPixel> {
        let geometry = self.get_geometry();
        TypedSize2D::new(geometry.width as f32, geometry.height as f32)
    }

    fn client_window(&self, _id: BrowserId) -> (Size2D<u32>, Point2D<i32>) {
        let geometry = self.get_geometry();

        (Size2D::new(geometry.width as u32, geometry.height as u32),
            Point2D::new(geometry.x as i32, geometry.y as i32))
    }

    fn set_page_title(&self, _id: BrowserId, title: Option<String>) {
        *self.title.borrow_mut() = title.clone();
        if let Some(ref callback) = *self.title_callback.borrow() {
            callback(title);
        }
    }

    fn allow_navigation(&self, _id: BrowserId, _url: ServoUrl, chan: ipc::IpcSender<bool>) {
        chan.send(true).ok();
    }

    fn set_inner_size(&self, _id: BrowserId, _size: Size2D<u32>) {
    }

    fn set_position(&self, _id: BrowserId, _point: Point2D<i32>) {
    }

    fn set_fullscreen_state(&self, _id: BrowserId, _state: bool) {
    }

    fn status(&self, _id: BrowserId, _status: Option<String>) {
    }

    fn load_start(&self, _id: BrowserId) {
    }

    fn load_end(&self, _id: BrowserId) {
    }

    fn load_error(&self, _id: BrowserId, _: NetError, _url: String) {
    }

    fn head_parsed(&self, _id: BrowserId) {
    }

    fn history_changed(&self, _id: BrowserId, entries: Vec<LoadData>, current: usize) {
        *self.can_go_back.borrow_mut() = !entries.is_empty() && current > 0;
        *self.can_go_forward.borrow_mut() = !entries.is_empty() && current < entries.len() - 1;
        let url = &entries[current].url;
        let url = url.as_str().to_string();
        *self.url.borrow_mut() = Some(url.clone());
        if let Some(ref callback) = *self.url_callback.borrow() {
            callback(url);
        }
    }

    fn set_cursor(&self, cursor: Cursor) {
        let cursor_name = match cursor {
            Cursor::None => "none",
            Cursor::Default => "default",
            Cursor::Pointer => "pointer",
            Cursor::ContextMenu => "context-menu",
            Cursor::Help => "help",
            Cursor::Progress => "progress",
            Cursor::Wait => "wait",
            Cursor::Cell => "cell",
            Cursor::Crosshair => "crosshair",
            Cursor::Text => "text",
            Cursor::VerticalText => "vertical-text",
            Cursor::Alias => "alias",
            Cursor::Copy => "copy",
            Cursor::Move => "move",
            Cursor::NoDrop => "no-drop",
            Cursor::NotAllowed => "not-allowed",
            Cursor::Grab => "grab",
            Cursor::Grabbing => "grabbing",
            Cursor::EResize => "e-resize",
            Cursor::NResize => "n-resize",
            Cursor::NeResize => "ne-resize",
            Cursor::NwResize => "nw-resize",
            Cursor::SResize => "s-resize",
            Cursor::SeResize => "se-resize",
            Cursor::SwResize => "sw-resize",
            Cursor::WResize => "w-resize",
            Cursor::EwResize => "ew-resize",
            Cursor::NsResize => "ns-resize",
            Cursor::NeswResize => "nesw-resize",
            Cursor::NwseResize => "nwse-resize",
            Cursor::ColResize => "col-resize",
            Cursor::RowResize => "row-resize",
            Cursor::AllScroll => "all-scroll",
            Cursor::ZoomIn => "zoom-in",
            Cursor::ZoomOut => "zoom-out",
        };
        let display = Display::get_default().unwrap();
        let cursor = gdk::Cursor::new_from_name(&display, cursor_name);
        let window = self.view.get_window().unwrap();
        window.set_cursor(&cursor);
    }

    fn set_favicon(&self, _id: BrowserId, _url: ServoUrl) {
    }

    fn handle_key(&self, _id: Option<BrowserId>, _ch: Option<char>, _key: Key, _mods: KeyModifiers) {
    }
}
