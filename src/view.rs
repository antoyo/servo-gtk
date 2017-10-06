use std::cell::RefCell;
use std::env;
use std::ptr;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use epoxy;
use gdk::{
    ScrollDirection,
    POINTER_MOTION_MASK,
    SCROLL_MASK,
};
use glib_itc::{Receiver, channel};
use gtk::{
    Continue,
    GLArea,
    GLAreaExt,
    Inhibit,
    WidgetExt,
};
use servo;
use servo::BrowserId;
use servo::compositing::windowing::{WindowEvent, WindowMethods};
use servo::euclid::{TypedPoint2D, TypedVector2D};
use servo::gl;
use servo::ipc_channel::ipc;
use servo::script_traits::TouchEventType;
use servo::servo_config::resource_files::set_resources_path;
use servo::servo_url::ServoUrl;
use shared_library::dynamic_library::DynamicLibrary;

use eventloop::GtkEventLoopWaker;
use window::GtkWindow;

macro_rules! with_servo {
    ($_self:ident, | $browser_id:ident, $servo:ident | $block:block) => {
        let mut state = $_self.state.borrow_mut();
        if let Some($browser_id) = state.browser_id.clone() {
            if let Some(ref mut servo) = state.servo {
                let mut $servo = servo.borrow_mut();
                $block
            }
        }
    };
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
    rx: Receiver,
    servo: Option<Rc<RefCell<servo::Servo<GtkWindow>>>>,
    view: View,
    window: Rc<GtkWindow>,
}

#[derive(Clone)]
pub struct WebView {
    state: Rc<RefCell<State>>,
}

impl WebView {
    pub fn new() -> Self {
        let view = GLArea::new();
        view.set_auto_render(false);
        view.set_has_depth_buffer(true);
        view.add_events((POINTER_MOTION_MASK | SCROLL_MASK).bits() as i32);
        view.set_vexpand(true); // TODO: put somewhere else?

        epoxy::load_with(|s| {
            unsafe {
                match DynamicLibrary::open(None).unwrap().symbol(s) {
                    Ok(v) => v,
                    Err(_) => ptr::null(),
                }
            }
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

    fn prepare(state: Rc<RefCell<State>>) {
        state.borrow().view.make_current();

        let servo = Rc::new(RefCell::new(servo::Servo::new(state.borrow().window.clone())));

        {
            let servo = servo.clone();
            state.borrow_mut().rx.connect_recv(move || {
                servo.borrow_mut().handle_events(vec![]);
                Continue(true)
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
            let inner_state = state.clone();
            let servo = servo.clone();
            state.borrow().view.connect_resize(move |_, _, _| {
                let event = WindowEvent::Resize(inner_state.borrow().window.framebuffer_size());
                servo.borrow_mut().handle_events(vec![event]);
            });
        }

        {
            let inner_state = state.clone();
            let servo = servo.clone();
            state.borrow().view.connect_scroll_event(move |_, event| {
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
                Inhibit(false)
            });
        }

        let path = env::current_dir().unwrap().join("resources");
        let path = path.to_str().unwrap().to_string();
        set_resources_path(Some(path));

        let url = ServoUrl::parse("https://servo.org").unwrap();
        let (sender, receiver) = ipc::channel().unwrap();
        servo.borrow_mut().handle_events(vec![WindowEvent::NewBrowser(url, sender)]);
        let browser_id = receiver.recv().unwrap();
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

    pub fn view(&self) -> View {
        self.state.borrow().view.clone()
    }
}
