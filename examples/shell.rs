/*
 * TODO: shortcut to go to next/previous tab.
 * TODO: close tab.
 * TODO: show if tab is loading.
 * TODO: favicon.
 * TODO: loading errors.
 */

extern crate gdk;
extern crate gtk;
extern crate servo_gtk;

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use gdk::{ScrollDirection, CONTROL_MASK};
use gdk::enums::key;
use gtk::{
    Button,
    ButtonExt,
    Cast,
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
use servo_gtk::view::View;

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

#[derive(Clone)]
struct Widgets {
    next_button: ToolButton,
    new_tab_button: Button,
    previous_button: ToolButton,
    reload_button: ToolButton,
    tabs: Notebook,
    url_entry: Entry,
    window: Window,
}

type WebViews = Rc<RefCell<Vec<WebView>>>;

struct App {
    webviews: WebViews,
    widgets: Rc<Widgets>,
}

impl App {
    fn new() -> Self {
        let app = Self::view();
        app.events();
        app
    }

    fn close_tab(tabs: &Notebook, webviews: &WebViews, widgets: &Widgets) {
        let page = tabs.get_current_page();
        tabs.remove_page(page);
        if let Some(page) = page {
            let webview = webviews.borrow_mut().remove(page as usize);
            if webviews.borrow().is_empty() {
                Self::new_tab(tabs, webviews, widgets);
            }
            webview.close();
        }
    }

    fn events(&self) {
        let url_entry = self.widgets.url_entry.clone();
        let tabs = self.widgets.tabs.clone();
        let webviews = self.webviews.clone();
        let widgets = self.widgets.clone();
        self.widgets.window.connect_key_press_event(move |_, event| {
            if event.get_state().contains(CONTROL_MASK) {
                match event.get_keyval() {
                    key::_0 => {
                        with_tab!(tabs, webviews, |webview| {
                            webview.reset_zoom();
                        });
                    },
                    key::l => url_entry.grab_focus(),
                    key::t => Self::new_tab(&tabs, &webviews, &widgets),
                    key::w => Self::close_tab(&tabs, &webviews, &widgets),
                    _ => (),
                }
            }
            Inhibit(false)
        });

        self.widgets.window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        let tabs = self.widgets.tabs.clone();
        let webviews = self.webviews.clone();
        self.widgets.url_entry.connect_activate(move |entry| {
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

        let tabs = self.widgets.tabs.clone();
        let webviews = self.webviews.clone();
        self.widgets.previous_button.connect_clicked(move |_| {
            with_tab!(tabs, webviews, |webview| {
                webview.back();
            });
        });

        let tabs = self.widgets.tabs.clone();
        let webviews = self.webviews.clone();
        self.widgets.next_button.connect_clicked(move |_| {
            with_tab!(tabs, webviews, |webview| {
                webview.forward();
            });
        });

        let tabs = self.widgets.tabs.clone();
        let webviews = self.webviews.clone();
        self.widgets.reload_button.connect_clicked(move |_| {
            with_tab!(tabs, webviews, |webview| {
                webview.reload();
            });
        });

        let tabs = self.widgets.tabs.clone();
        let webviews = self.webviews.clone();
        let widgets = self.widgets.clone();
        self.widgets.new_tab_button.connect_clicked(move |_| {
            Self::new_tab(&tabs, &webviews, &widgets);
        });

        let previous_button = self.widgets.previous_button.clone();
        let next_button = self.widgets.next_button.clone();
        let window = self.widgets.window.clone();
        let webviews = self.webviews.clone();
        let url_entry = self.widgets.url_entry.clone();
        self.widgets.tabs.connect_switch_page(move |_, _, page| {
            let webviews = webviews.borrow();
            if let Some(webview) = webviews.get(page as usize) {
                let url = webview.get_url().unwrap_or_default();
                url_entry.set_text(&url);

                let title = webview.get_title().unwrap_or_else(|| "Servo Shell".to_string());
                window.set_title(&title);

                previous_button.set_sensitive(webview.can_go_back());
                next_button.set_sensitive(webview.can_go_forward());
            }
        });
    }

    fn new_tab(tabs: &Notebook, webviews: &WebViews, widgets: &Widgets) {
        let webview = WebView::new();
        let view = webview.view();
        view.set_vexpand(true);
        tabs.add(&view);
        tabs.set_tab_label_text(&view, "New tab");
        view.show();
        Self::webview_events(&widgets, &webview);
        webviews.borrow_mut().push(webview);
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

        window.show_all();

        let widgets = Rc::new(Widgets {
            next_button,
            new_tab_button,
            previous_button,
            reload_button,
            tabs,
            url_entry,
            window,
        });

        Self::webview_events(&widgets, &webview);

        let app = App {
            webviews: Rc::new(RefCell::new(vec![webview])),
            widgets: widgets.clone(),
        };

        app
    }

    fn webview_events(widgets: &Widgets, webview: &WebView) {
        {
            let tabs = widgets.tabs.clone();
            let window = widgets.window.clone();
            let view = webview.view();
            webview.connect_title_changed(move |page_title| {
                let title: Cow<str> = match page_title {
                    Some(ref title) => format!("{} - Servo Shell", title).into(),
                    None => "Servo Shell".into(),
                };
                if current_tab_active(&tabs, &view) {
                    window.set_title(&title);
                }
                let title = page_title.as_ref().map(String::as_str).unwrap_or("(no title)");
                tabs.set_tab_label_text(&view, title);
            });
        }

        {
            let view = webview.clone();
            webview.view().connect_scroll_event(move |_, event| {
                if event.get_state().contains(CONTROL_MASK) {
                    let step = match event.get_direction() {
                        ScrollDirection::Down => -0.1,
                        ScrollDirection::Up => 0.1,
                        _ => return Inhibit(false),
                    };

                    view.zoom(step);
                }

                Inhibit(false)
            });
        }

        {
            let previous_button = widgets.previous_button.clone();
            let next_button = widgets.next_button.clone();
            let tabs = widgets.tabs.clone();
            let wview = webview.clone();
            let view = webview.view();
            let url_entry = widgets.url_entry.clone();
            webview.connect_url_changed(move |url| {
                if current_tab_active(&tabs, &view) {
                    previous_button.set_sensitive(wview.can_go_back());
                    next_button.set_sensitive(wview.can_go_forward());
                    url_entry.set_text(&url);
                }
            });
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

fn current_tab_active(tabs: &Notebook, view: &View) -> bool {
    tabs.get_nth_page(tabs.get_current_page()) == Some(view.clone().upcast())
}
