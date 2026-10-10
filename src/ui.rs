use crate::catalog::{self, Tool};
use crate::locale::{self, Language};
use gtk::{glib, prelude::*};
use std::{cell::Cell, rc::Rc};

pub struct Ui {
    pub window: gtk::ApplicationWindow,
    pub selector: gtk::DropDown,
    pub status: gtk::Label,
    pub language: Rc<Cell<Language>>,
}

pub fn build(app: &gtk::Application, tools: Rc<Vec<Tool>>) -> Ui {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("style.css"));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
    let initial_language = locale::selected_language();
    let language = Rc::new(Cell::new(initial_language));
    let window = gtk::ApplicationWindow::builder().application(app)
        .title(initial_language.text("windowTitle")).default_width(560).default_height(440)
        .decorated(false).build();
    window.set_icon_name(Some("clavis"));
    window.add_css_class("overlay");
    window.connect_map(|_| eprintln!("Overlay mapped by GTK"));
    window.connect_unmap(|_| eprintln!("Overlay hidden"));
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    body.set_margin_top(12); body.set_margin_bottom(10);
    body.set_margin_start(12); body.set_margin_end(12);
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let names: Vec<&str> = tools.iter().map(|tool| tool.name.as_str()).collect();
    let selector = gtk::DropDown::from_strings(&names);
    selector.set_tooltip_text(Some(initial_language.text("toolSelector")));
    selector.set_hexpand(true);
    selector.set_selected(catalog::selected_tool(&tools));
    header.append(&selector);
    let kind = gtk::Label::new(None); kind.add_css_class("muted"); header.append(&kind);
    let language_selector = gtk::DropDown::from_strings(&["English", "Español"]);
    language_selector.set_selected(initial_language.index());
    language_selector.set_size_request(98, -1);
    language_selector.set_tooltip_text(Some(initial_language.text("language")));
    header.append(&language_selector);
    let hide = gtk::Button::from_icon_name("window-close-symbolic");
    hide.set_tooltip_text(Some(initial_language.text("hide"))); hide.add_css_class("flat");
    let weak = window.downgrade();
    hide.connect_clicked(move |_| { if let Some(window) = weak.upgrade() { window.set_visible(false); } });
    header.append(&hide); body.append(&header);
    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some(initial_language.text("search")));
    body.append(&search);
    let list = gtk::Grid::builder().column_homogeneous(true).column_spacing(16).row_spacing(2).build();
    let scroll = gtk::ScrolledWindow::builder().vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never).child(&list).build();
    body.append(&scroll);
    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let status = gtk::Label::new(Some(initial_language.text("status")));
    status.add_css_class("muted"); status.set_hexpand(true); status.set_xalign(0.0);
    status.set_wrap(true); footer.append(&status);
    let docs = gtk::LinkButton::with_label("https://ghostty.org", initial_language.text("docs"));
    docs.add_css_class("flat"); footer.append(&docs); body.append(&footer);
    window.set_child(Some(&body));

    let docs_for_render = docs.clone();
    let render: Rc<dyn Fn(u32, &str)> = {
        let tools = tools.clone(); let status = status.clone();
        let language = language.clone();
        Rc::new(move |index, query| {
            while let Some(child) = list.first_child() { list.remove(&child); }
            let Some(tool) = tools.get(index as usize) else { return; };
            let locale = language.get();
            kind.set_text(if tool.kind == "Keys" { locale.text("keys") } else { &tool.kind });
            docs_for_render.set_uri(&tool.source);
            docs_for_render.set_tooltip_text(Some(&tool.note));
            status.set_text(locale.text("status"));
            let copied_message = locale.text("copied");
            let mut count = 0;
            for entry in tool.entries.iter().filter(|entry| catalog::matches(entry, query)) {
                count += 1;
                let button = gtk::Button::new(); button.add_css_class("reference-row");
                button.set_tooltip_text(Some(&format!("{}\n{}\n{}", entry.key, entry.action, locale.text("copyHint"))));
                button.set_hexpand(true);
                let row = gtk::Box::new(gtk::Orientation::Vertical, 2);
                let key = gtk::Label::new(Some(&entry.key));
                key.add_css_class("key"); key.set_xalign(0.0);
                key.set_wrap(true); key.set_wrap_mode(gtk::pango::WrapMode::WordChar);
                key.set_max_width_chars(32);
                let action = gtk::Label::new(Some(&entry.action));
                action.set_xalign(0.0); action.set_wrap(true); action.set_hexpand(true);
                action.add_css_class("muted");
                row.append(&key); row.append(&action); button.set_child(Some(&row));
                let value = entry.key.clone(); let status = status.clone();
                button.connect_clicked(move |button| {
                    button.clipboard().set_text(&value);
                    status.set_text(copied_message);
                });
                // Row-major order matches keyboard navigation and search results.
                let (column, row) = cell_position(count - 1);
                list.attach(&button, column, row, 1, 1);
            }
            if count == 0 {
                let empty = gtk::Label::new(Some(locale.text("empty")));
                empty.add_css_class("muted"); empty.set_margin_top(24); list.attach(&empty, 0, 0, 2, 1);
            }
        })
    };
    render(selector.selected(), "");
    let update = render.clone(); let weak_search = search.downgrade();
    selector.connect_selected_notify(move |selector| {
        if let Some(search) = weak_search.upgrade() {
            update(selector.selected(), &search.text());
        }
        if let Some(tool) = tools.get(selector.selected() as usize) { catalog::remember_tool(&tool.name); }
    });
    let weak_selector = selector.downgrade();
    let search_render = render.clone();
    search.connect_search_changed(move |search| {
        if let Some(selector) = weak_selector.upgrade() { search_render(selector.selected(), &search.text()); }
    });
    let update = render.clone();
    let weak_search = search.downgrade();
    let weak_window = window.downgrade();
    let tool_selector = selector.clone();
    let hide_button = hide.clone();
    let docs_button = docs.clone();
    let active_language = language.clone();
    language_selector.connect_selected_notify(move |selector| {
        let selected_language = Language::from_index(selector.selected());
        active_language.set(selected_language);
        locale::remember_language(selected_language);
        if let Some(window) = weak_window.upgrade() {
            window.set_title(Some(selected_language.text("windowTitle")));
        }
        tool_selector.set_tooltip_text(Some(selected_language.text("toolSelector")));
        hide_button.set_tooltip_text(Some(selected_language.text("hide")));
        selector.set_tooltip_text(Some(selected_language.text("language")));
        docs_button.set_label(selected_language.text("docs"));
        if let Some(search) = weak_search.upgrade() {
            search.set_placeholder_text(Some(selected_language.text("search")));
            update(tool_selector.selected(), &search.text());
        }
    });
    window.connect_close_request(|window| { window.set_visible(false); glib::Propagation::Stop });
    let keys = gtk::EventControllerKey::new(); let weak = window.downgrade();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gtk::gdk::Key::Escape {
            if let Some(window) = weak.upgrade() { window.set_visible(false); }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(keys);
    Ui { window, selector, status, language }
}

fn cell_position(index: usize) -> (i32, i32) {
    ((index % 2) as i32, (index / 2) as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn references_flow_in_two_columns() {
        assert_eq!(cell_position(0), (0, 0));
        assert_eq!(cell_position(1), (1, 0));
        assert_eq!(cell_position(2), (0, 1));
        assert_eq!(cell_position(14), (0, 7));
    }
}

pub fn capture(window: &gtk::ApplicationWindow, path: &str) -> Result<(), String> {
    let surface = window.surface().ok_or("Window has no surface")?;
    let renderer = gtk::gsk::Renderer::for_surface(&surface).ok_or("Cannot create renderer")?;
    let paintable = gtk::WidgetPaintable::new(Some(window));
    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(&snapshot, window.width() as f64, window.height() as f64);
    let node = snapshot.to_node().ok_or("No rendered content")?;
    let texture = renderer.render_texture(&node, None);
    let result = texture.save_to_png(path).map_err(|error| error.to_string());
    renderer.unrealize();
    result
}
