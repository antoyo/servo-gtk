/*
 * TODO: show if tab is loading.
 * TODO: update url in entry.
 * TODO: zoom.
 * TODO: favicon.
 * TODO: handle history changed to enable/disable back/forward buttons.
 * TODO: loading errors.
 * TODO: show title on tabs.
 */

extern crate gdk;
extern crate gtk;
extern crate servo_gtk;

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::{
    Button,
    ButtonExt,
    ContainerExt,
    Entry,
    EntryExt,
    Image,
    Inhibit,
    Notebook,
    NotebookExt,
    NotebookExtManual,
    PackType,
    SeparatorToolItem,
    Toolbar,
    ToolButton,
    ToolButtonExt,
    ToolItem,
    ToolItemExt,
    WidgetExt,
    Window,
    WindowExt,
    WindowType,
};
use gtk::Orientation::Vertical;
use servo_gtk::WebView;

macro_rules! with_tab {
    ($tabs:expr, $webviews:expr, |$webview:ident| $block:block) => {
        if let Some(page) = $tabs.get_current_page() {
            let webviews = $webviews.borrow();
            if let Some($webview) = webviews.get(page as usize) {
                $block
            }
        }
        // TODO: handle errors.
    };
}

struct App {
    next_button: ToolButton,
    new_tab_button: Button,
    previous_button: ToolButton,
    reload_button: ToolButton,
    tabs: Notebook,
    url_entry: Entry,
    webviews: Rc<RefCell<Vec<WebView>>>,
    window: Window,
}

impl App {
    fn new() -> Self {
        let app = Self::view();
        app.events();
        app
    }

    fn events(&self) {
        self.window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        let tabs = self.tabs.clone();
        let webviews = self.webviews.clone();
        self.url_entry.connect_activate(move |entry| {
            let url = entry.get_text().unwrap();
            let url =
                if url.contains("://") {
                    url.to_string()
                }
                else {
                    format!("http://{}", url)
                };
            with_tab!(tabs, webviews, |webview| {
                webview.load(&url);
            });
        });

        let tabs = self.tabs.clone();
        let webviews = self.webviews.clone();
        self.previous_button.connect_clicked(move |_| {
            with_tab!(tabs, webviews, |webview| {
                webview.back();
            });
        });

        let tabs = self.tabs.clone();
        let webviews = self.webviews.clone();
        self.next_button.connect_clicked(move |_| {
            with_tab!(tabs, webviews, |webview| {
                webview.forward();
            });
        });

        let tabs = self.tabs.clone();
        let webviews = self.webviews.clone();
        self.reload_button.connect_clicked(move |_| {
            with_tab!(tabs, webviews, |webview| {
                webview.reload();
            });
        });

        let tabs = self.tabs.clone();
        let webviews = self.webviews.clone();
        self.new_tab_button.connect_clicked(move |_| {
            let webview = WebView::new();
            let view = webview.view();
            view.set_vexpand(true);
            tabs.add(&view);
            tabs.set_tab_label_text(&view, "Tab");
            view.show();
            webviews.borrow_mut().push(webview);
        });
    }

    fn view() -> App {
        let window = Window::new(WindowType::Toplevel);
        window.set_size_request(800, 600);

        let vbox = gtk::Box::new(Vertical, 0);
        window.add(&vbox);

        let toolbar = Toolbar::new();
        vbox.add(&toolbar);

        let previous_button = ToolButton::new(&icon("go-previous"), None);
        toolbar.add(&previous_button);

        let next_button = ToolButton::new(&icon("go-next"), None);
        toolbar.add(&next_button);

        toolbar.add(&SeparatorToolItem::new());

        let reload_button = ToolButton::new(&icon("view-refresh"), None);
        toolbar.add(&reload_button);

        toolbar.add(&SeparatorToolItem::new());

        let url_entry = Entry::new();
        let url_tool_item = ToolItem::new();
        url_tool_item.set_expand(true);
        url_tool_item.add(&url_entry);
        toolbar.add(&url_tool_item);

        let tabs = Notebook::new();
        let new_tab_button = Button::new();
        new_tab_button.add(&icon("list-add"));
        new_tab_button.show_all();
        tabs.set_action_widget(&new_tab_button, PackType::End);
        vbox.add(&tabs);

        let webview = WebView::new();
        let view = webview.view();
        view.set_vexpand(true);
        tabs.add(&view);

        {
            let window = window.clone();
            webview.connect_title_changed(move |title| {
                let title: Cow<str> = match title {
                    Some(ref title) => format!("{} - Servo Shell", title).into(),
                    None => "Servo Shell".into(),
                };
                window.set_title(&title);
            });
        }

        window.show_all();

        App {
            next_button,
            new_tab_button,
            previous_button,
            reload_button,
            tabs,
            url_entry,
            webviews: Rc::new(RefCell::new(vec![webview])),
            window,
        }
    }
}

fn main() {
    gtk::init().unwrap();

    let _app = App::new();

    gtk::main();
}

fn icon(name: &str) -> Image {
    Image::new_from_file(format!("images/{}.png", name))
}
