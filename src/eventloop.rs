use std::sync::{Arc, Mutex};

use glib_itc::Sender;
use servo::compositing::compositor_thread::EventLoopWaker;

pub struct GtkEventLoopWaker {
    tx: Arc<Mutex<Sender>>,
}

impl GtkEventLoopWaker {
    pub fn new(tx: Arc<Mutex<Sender>>) -> Self {
        GtkEventLoopWaker {
            tx,
        }
    }
}

impl EventLoopWaker for GtkEventLoopWaker {
    // Use by servo to share the "event loop waker" across threads
    fn clone(&self) -> Box<EventLoopWaker + Send> {
        Box::new(GtkEventLoopWaker {
            tx: self.tx.clone(),
        })
    }

    // Called by servo when the main thread needs to wake up
    fn wake(&self) {
        self.tx.lock().unwrap().send();
    }
}
