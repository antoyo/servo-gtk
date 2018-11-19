use std::cell::{Cell, RefCell};
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::ptr;
use std::rc::Rc;
use std::sync::{Arc, Mutex, Once, ONCE_INIT};

use epoxy;
use gdk::{
    EventMask,
    ScrollDirection,
};
use gdk::ModifierType;
use glib_itc::{Receiver, channel};
use gtk::{
    Cast,
    Continue,
    GLArea,
    GLAreaExt,
    Inhibit,
    IsA,
    WidgetExt,
};
use keyboard_types::{Key, KeyboardEvent};
use servo;
use servo::BrowserId;
use servo::compositing::windowing::{MouseWindowEvent, WindowEvent};
use servo::embedder_traits::resources::{self, Resource};
use servo::euclid::{TypedPoint2D, TypedVector2D};
use servo::gl;
use servo::ipc_channel::ipc;
use servo::msg::constellation_msg::{TraversalDirection};
use servo::script_traits::TouchEventType;
use servo::servo_url::ServoUrl;
use shared_library::dynamic_library::DynamicLibrary;

use convert;
use eventloop::GtkEventLoopWaker;
use window::GtkWindow;

macro_rules! with_servo {
    ($_self:ident, | $browser_id:ident, $servo:ident | $block:block, no_activate) => {
        let mut state = $_self.state.borrow_mut();
        if let Some($browser_id) = state.browser_id.clone() {
            if let Some(ref mut servo) = state.servo {
                let mut $servo = servo.borrow_mut();
                $block
            }
        }
    };
    ($_self:ident, | $browser_id:ident, $servo:ident | $block:block) => {
        $_self.activate();
        with_servo!($_self, |$browser_id, $servo| $block, no_activate);
    };
}

static EPOXY_INIT: Once = ONCE_INIT;

struct ResourceReader;

impl resources::ResourceReaderMethods for ResourceReader {
    fn read(&self, file: Resource) -> Vec<u8> {
        let file = filename(file);
        let mut path = resources_dir_path().expect("Can't find resources directory");
        path.push(file);
        fs::read(path).expect("Can't read file")
    }
    fn sandbox_access_files_dirs(&self) -> Vec<PathBuf> {
        vec![resources_dir_path().expect("Can't find resources directory")]
    }
    fn sandbox_access_files(&self) -> Vec<PathBuf> {
        vec![]
    }
}

fn filename(file: Resource) -> &'static str {
    match file {
        Resource::Preferences => "prefs.json",
        Resource::BluetoothBlocklist => "gatt_blocklist.txt",
        Resource::DomainList => "public_domains.txt",
        Resource::HstsPreloadList => "hsts_preload.json",
        Resource::SSLCertificates => "certs",
        Resource::BadCertHTML => "badcert.html",
        Resource::NetErrorHTML => "neterror.html",
        Resource::UserAgentCSS => "user-agent.css",
        Resource::ServoCSS => "servo.css",
        Resource::PresentationalHintsCSS => "presentational-hints.css",
        Resource::QuirksModeCSS => "quirks-mode.css",
        Resource::RippyPNG => "rippy.png",
    }
}

fn resources_dir_path() -> io::Result<PathBuf> {
    let path = env::current_dir().unwrap().join("resources");
    Ok(path)
}

pub type View = GLArea;

struct Pos {
    x: f64,
    y: f64,
}

impl Pos {
    fn new(x: f64, y: f64) -> Self {
        Pos {
            x,
            y,
        }
    }
}

struct State {
    browser_id: Option<BrowserId>,
    pointer: Pos,
    rx: Receiver<()>,
    servo: Option<Rc<RefCell<servo::Servo<GtkWindow>>>>,
    view: View,
    window: Rc<GtkWindow>,
    zoom_level: Cell<f32>,
}

#[derive(Clone)]
pub struct WebView {
    state: Rc<RefCell<State>>,
}

impl WebView {
    pub fn new<V: IsA<GLArea> + IsA<gtk::Object>>(view: V) -> Self {
        let view: GLArea = view.upcast();
        view.set_auto_render(false);
        view.set_has_depth_buffer(true);
        view.add_events((EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK | EventMask::POINTER_MOTION_MASK
            | EventMask::SCROLL_MASK).bits() as i32);
        view.set_can_focus(true);
        view.set_size_request(200, 200);

        EPOXY_INIT.call_once(|| {
            epoxy::load_with(|s| {
                unsafe {
                    match DynamicLibrary::open(None).unwrap().symbol(s) {
                        Ok(v) => v,
                        Err(_) => ptr::null(),
                    }
                }
            });
        });

        let gl = unsafe {
            gl::GlFns::load_with(epoxy::get_proc_addr)
        };

        let (tx, rx) = channel();

        let waker = Box::new(GtkEventLoopWaker::new(Arc::new(Mutex::new(tx))));

        let window = Rc::new(GtkWindow::new(gl, view.clone(), waker));

        let state = Rc::new(RefCell::new(State {
            browser_id: None,
            pointer: Pos::new(0.0, 0.0),
            rx,
            servo: None,
            view: view.clone(),
            window,
            zoom_level: Cell::new(1.0),
        }));

        {
            let state = state.clone();
            view.connect_realize(move |_| {
                Self::prepare(state.clone());
            });
        }

        WebView {
            state,
        }
    }

    fn activate(&self) {
        // FIXME: can we avoid calling this method everytime?
        with_servo!(self, |browser_id, servo| {
            let event = WindowEvent::SelectBrowser(browser_id);
            servo.handle_events(vec![event]);
        }, no_activate);
    }

    pub fn back(&self) {
        with_servo!(self, |browser_id, servo| {
            let event = WindowEvent::Navigation(browser_id, TraversalDirection::Back(1));
            servo.handle_events(vec![event]);
        });
    }

    pub fn can_go_back(&self) -> bool {
        let state = self.state.borrow();
        state.window.can_go_back()
    }

    pub fn can_go_forward(&self) -> bool {
        let state = self.state.borrow();
        state.window.can_go_forward()
    }

    pub fn close(&self) {
        // FIXME: warning.
        // FIXME: should change the url (i.e. because it triggers page switch).
        with_servo!(self, |browser_id, servo| {
            let event = WindowEvent::CloseBrowser(browser_id);
            servo.handle_events(vec![event]);
        });
    }

    pub fn connect_title_changed<F: Fn(Option<String>) + 'static>(&self, callback: F) {
        let state = self.state.borrow();
        state.window.connect_title_changed(callback);
    }

    pub fn connect_url_changed<F: Fn(String) + 'static>(&self, callback: F) {
        let state = self.state.borrow();
        state.window.connect_url_changed(callback);
    }

    pub fn forward(&self) {
        with_servo!(self, |browser_id, servo| {
            let event = WindowEvent::Navigation(browser_id, TraversalDirection::Forward(1));
            servo.handle_events(vec![event]);
        });
    }

    pub fn get_title(&self) -> Option<String> {
        let state = self.state.borrow();
        state.window.get_title()
    }

    pub fn get_url(&self) -> Option<String> {
        let state = self.state.borrow();
        state.window.get_url()
    }

    pub fn get_zoom(&self) -> f32 {
        let state = self.state.borrow();
        state.zoom_level.get()
    }

    pub fn load(&self, url: &str) {
        with_servo!(self, |browser_id, servo| {
            match ServoUrl::parse(url) {
                Ok(url) => {
                    let event = WindowEvent::LoadUrl(browser_id, url);
                    servo.handle_events(vec![event]);
                },
                // TODO: return an error.
                Err(error) => println!("Error: {}", error),
            }
        });
    }

    fn prepare(state: Rc<RefCell<State>>) {
        state.borrow().view.make_current();

        resources::set(Box::new(ResourceReader));

        let servo = Rc::new(RefCell::new(servo::Servo::new(state.borrow().window.clone())));

        {
            let servo = servo.clone();
            state.borrow_mut().rx.connect_recv(move |()| {
                servo.borrow_mut().handle_events(vec![]);
                Continue(true)
            });
        }

        {
            let servo = servo.clone();
            state.borrow().view.connect_key_press_event(move |_, event| {
                let (char, key) = convert::key(event.get_keyval());
                if let Some(key) = key {
                    let modifiers = convert::modifiers(event.get_state());
                    let mut event = KeyboardEvent::default();
                    event.key = key;
                    event.modifiers = modifiers;
                    let event = WindowEvent::Keyboard(event);
                    servo.borrow_mut().handle_events(vec![event]);
                }
                Inhibit(false)
            });
        }

        /*{
            let servo = servo.clone();
            state.borrow().view.connect_key_release_event(move |_, event| {
                let (char, key) = convert::key(event.get_keyval());
                if let Some(key) = key {
                    let modifiers = convert::modifiers(event.get_state());
                    let event = WindowEvent::KeyEvent(char, key, KeyState::Released, modifiers);
                    servo.borrow_mut().handle_events(vec![event]);
                }
                Inhibit(false)
            });
        }*/

        {
            let servo = servo.clone();
            let view = state.borrow().view.clone();
            state.borrow().view.connect_button_press_event(move |_, event| {
                view.grab_focus();
                let (x, y) = event.get_position();
                let event = WindowEvent::MouseWindowEventClass(MouseWindowEvent::MouseDown(
                        convert::mouse_button(event.get_button()), TypedPoint2D::new(x as f32, y as f32)));
                servo.borrow_mut().handle_events(vec![event]);
                Inhibit(false)
            });
        }

        {
            let servo = servo.clone();
            state.borrow().view.connect_button_release_event(move |_, event| {
                let (x, y) = event.get_position();
                let button = convert::mouse_button(event.get_button());
                let event = WindowEvent::MouseWindowEventClass(MouseWindowEvent::MouseUp(
                        button, TypedPoint2D::new(x as f32, y as f32)));
                servo.borrow_mut().handle_events(vec![event]);
                let event = WindowEvent::MouseWindowEventClass(MouseWindowEvent::Click(
                        button, TypedPoint2D::new(x as f32, y as f32)));
                servo.borrow_mut().handle_events(vec![event]); // TODO: check if it is the right place to trigger this event.
                Inhibit(false)
            });
        }

        {
            let inner_state = state.clone();
            let servo = servo.clone();
            state.borrow().view.connect_motion_notify_event(move |_, event| {
                let (x, y) = event.get_position();
                let pointer = &mut inner_state.borrow_mut().pointer;
                pointer.x = x;
                pointer.y = y;
                let event = WindowEvent::MouseWindowMoveEventClass(TypedPoint2D::new(x as f32, y as f32));
                servo.borrow_mut().handle_events(vec![event]);
                Inhibit(false)
            });
        }

        {
            let servo = servo.clone();
            state.borrow().view.connect_resize(move |_, _, _| {
                servo.borrow_mut().handle_events(vec![WindowEvent::Resize]);
            });
        }

        {
            let inner_state = state.clone();
            let servo = servo.clone();
            state.borrow().view.connect_scroll_event(move |_, event| {
                let state = event.get_state();
                if !state.contains(ModifierType::CONTROL_MASK) {
                    let phase = match event.get_direction() {
                        ScrollDirection::Down => TouchEventType::Down,
                        ScrollDirection::Up => TouchEventType::Up,
                        ScrollDirection::Left => TouchEventType::Cancel, // FIXME
                        ScrollDirection::Right => TouchEventType::Cancel, // FIXME
                        ScrollDirection::Smooth | _ => TouchEventType::Cancel, // FIXME
                    };
                    let dx = 0.0;
                    let dy =
                        match phase {
                            TouchEventType::Up => 1.0,
                            TouchEventType::Down => -1.0,
                            _ => 0.0,
                        };
                    let dy = dy * 38.0;
                    let pointer = {
                        let pointer = &inner_state.borrow().pointer;
                        TypedPoint2D::new(pointer.x as i32, pointer.y as i32)
                    };
                    let scroll_location = servo::webrender_api::ScrollLocation::Delta(TypedVector2D::new(dx as f32, dy as f32));
                    let event = WindowEvent::Scroll(scroll_location, pointer, phase);
                    servo.borrow_mut().handle_events(vec![event]);
                }
                Inhibit(false)
            });
        }

        let url = ServoUrl::parse("https://servo.org").unwrap();
        let browser_id = BrowserId::new();
        servo.borrow_mut().handle_events(vec![WindowEvent::NewBrowser(url, browser_id)]);
        servo.borrow_mut().handle_events(vec![WindowEvent::SelectBrowser(browser_id)]);
        state.borrow_mut().browser_id = Some(browser_id);
        state.borrow_mut().servo = Some(servo);
    }

    pub fn reload(&self) {
        with_servo!(self, |browser_id, servo| {
            let event = WindowEvent::Reload(browser_id);
            servo.handle_events(vec![event]);
        });
    }

    pub fn reset_zoom(&self) {
        {
            let state = self.state.borrow();
            state.zoom_level.set(1.0);
        }
        with_servo!(self, |_browser_id, servo| {
            servo.handle_events(vec![WindowEvent::ResetZoom]);
        });
    }

    pub fn zoom(&self, step: f32) {
        let step = step + 1.0;
        {
            let state = self.state.borrow();
            state.zoom_level.set(state.zoom_level.get() * step);
        }
        with_servo!(self, |_browser_id, servo| {
            servo.handle_events(vec![WindowEvent::Zoom(step)]);
        });
    }

    pub fn view(&self) -> View {
        self.state.borrow().view.clone()
    }
}
