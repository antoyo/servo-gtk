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
use servo::compositing::windowing::{WindowEvent, WindowMethods};
use servo::euclid::{TypedPoint2D, TypedVector2D};
use servo::gl;
use servo::ipc_channel::ipc;
use servo::script_traits::TouchEventType;
use servo::servo_config::resource_files::set_resources_path;
use servo::servo_url::ServoUrl;
use shared_library::dynamic_library::DynamicLibrary;

use controller::WebViewController;
use eventloop::GtkEventLoopWaker;
use window::GtkWindow;

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

pub struct WebView {
    pointer: Rc<RefCell<Pos>>,
    rx: Receiver,
    view: View,
    window: Rc<GtkWindow>,
}

impl WebView {
    pub fn new() -> Self {
        let view = GLArea::new();
        view.set_auto_render(false);
        view.set_has_depth_buffer(true);
        view.add_events((POINTER_MOTION_MASK | SCROLL_MASK).bits() as i32);
        view.set_vexpand(true); // TODO: put somewhere else?

        let pointer = Rc::new(RefCell::new(Pos::new(0.0, 0.0)));

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

        WebView {
            pointer,
            rx,
            view,
            window,
        }
    }

    pub fn show(&mut self) -> WebViewController {
        self.view.make_current();

        let servo = Rc::new(RefCell::new(servo::Servo::new(self.window.clone())));

        {
            let servo = servo.clone();
            self.rx.connect_recv(move || {
                servo.borrow_mut().handle_events(vec![]);
                Continue(true)
            });
        }

        {
            let pointer = self.pointer.clone();
            let servo = servo.clone();
            self.view.connect_motion_notify_event(move |_, event| {
                let (x, y) = event.get_position();
                let mut pointer = pointer.borrow_mut();
                pointer.x = x;
                pointer.y = y;
                let event = WindowEvent::MouseWindowMoveEventClass(TypedPoint2D::new(x as f32, y as f32));
                servo.borrow_mut().handle_events(vec![event]);
                Inhibit(false)
            });
        }

        {
            let servo = servo.clone();
            let window = self.window.clone();
            self.view.connect_resize(move |_, _, _| {
                let event = WindowEvent::Resize(window.framebuffer_size());
                servo.borrow_mut().handle_events(vec![event]);
            });
        }

        {
            let pointer = self.pointer.clone();
            let servo = servo.clone();
            self.view.connect_scroll_event(move |_, event| {
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
                    let pointer = pointer.borrow();
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

        WebViewController::new(servo, browser_id)
    }

    pub fn view(&self) -> &View {
        &self.view
    }
}
