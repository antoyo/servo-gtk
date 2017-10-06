use std::cell::RefCell;
use std::rc::Rc;

use servo;
use servo::BrowserId;
use servo::compositing::windowing::WindowEvent;

use window::GtkWindow;

#[derive(Clone)]
pub struct WebViewController {
    browser_id: BrowserId,
    servo: Rc<RefCell<servo::Servo<GtkWindow>>>,
}

impl WebViewController {
    pub fn new(servo: Rc<RefCell<servo::Servo<GtkWindow>>>, browser_id: BrowserId) -> Self {
        WebViewController {
            browser_id,
            servo,
        }
    }

    pub fn reload(&self) {
        let event = WindowEvent::Reload(self.browser_id);
        self.servo.borrow_mut().handle_events(vec![event]);
    }
}
