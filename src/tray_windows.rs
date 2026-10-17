use crate::OverlayEvent;
use gtk::glib;
use std::{cell::RefCell, sync::mpsc::Sender, time::Duration};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
    Icon, TrayIcon, TrayIconBuilder,
};

thread_local! {
    static TRAY: RefCell<Option<TrayIcon>> = const { RefCell::new(None) };
}

pub fn start(sender: Sender<OverlayEvent>, names: Vec<String>) {
    let menu = Menu::new();
    let show = MenuItem::with_id("show", "Show reference", true, None);
    let hide = MenuItem::with_id("hide", "Hide reference", true, None);
    let tools = Submenu::new("Tools", true);
    let mut tool_items = Vec::new();
    for (index, name) in names.iter().enumerate() {
        let item = MenuItem::with_id(format!("tool-{index}"), name, true, None);
        if let Err(error) = tools.append(&item) { eprintln!("Cannot add Clavis tray menu item: {error}"); }
        tool_items.push(item.id().clone());
    }
    let separator = PredefinedMenuItem::separator();
    let quit = MenuItem::with_id("quit", "Quit", true, None);
    for result in [menu.append(&show), menu.append(&hide), menu.append(&tools), menu.append(&separator), menu.append(&quit)] {
        if let Err(error) = result { eprintln!("Cannot add Clavis tray menu item: {error}"); }
    }

    let rgba = include_bytes!("../packaging/clavis-tray.argb").chunks_exact(4)
        .flat_map(|argb| [argb[1], argb[2], argb[3], argb[0]])
        .collect::<Vec<_>>();
    let icon = match Icon::from_rgba(rgba, 32, 32) {
        Ok(icon) => icon,
        Err(error) => { eprintln!("Cannot load Clavis tray icon: {error}"); return; }
    };
    match TrayIconBuilder::new().with_menu(Box::new(menu)).with_tooltip("Clavis · Terminal reference")
        .with_icon(icon).build() {
        Ok(tray) => TRAY.with(|slot| *slot.borrow_mut() = Some(tray)),
        Err(error) => { eprintln!("Cannot start Clavis system tray: {error}"); return; }
    }

    let show_id = show.id().clone();
    let hide_id = hide.id().clone();
    let quit_id = quit.id().clone();
    let menu_events = MenuEvent::receiver();
    glib::timeout_add_local(Duration::from_millis(30), move || {
        for event in menu_events.try_iter() {
            if event.id == show_id { let _ = sender.send(OverlayEvent::Show); }
            else if event.id == hide_id { let _ = sender.send(OverlayEvent::Hide); }
            else if event.id == quit_id { let _ = sender.send(OverlayEvent::Quit); }
            else if let Some(index) = tool_items.iter().position(|id| *id == event.id) {
                let _ = sender.send(OverlayEvent::SelectTool(index as u32));
            }
        }
        glib::ControlFlow::Continue
    });
}
