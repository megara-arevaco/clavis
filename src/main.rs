use gtk::{glib, prelude::*};
use serde::Deserialize;
use std::{cell::Cell, collections::HashMap, env, fs, path::PathBuf, rc::Rc, sync::mpsc, thread, time::{Duration, Instant}};

mod input;
mod catalog;
mod ui;
mod tray;

enum OverlayEvent { Toggle, DoubleTap, Show, Hide, SelectTool(u32), Quit, Error(String), InputNotice(String) }

#[derive(Deserialize)]
struct Shortcut {
    key: String,
    action: String,
}

fn shortcuts() -> Vec<Shortcut> {
    let base = env::var_os("XDG_CONFIG_HOME").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".config"));
    let mut entries = Vec::new();
    if let Ok(config) = fs::read_to_string(base.join("ghostty/config")) {
        for line in config.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            if let Some((name, binding)) = line.split_once('=') {
                if name.trim() != "keybind" { continue; }
                if let Some((key, action)) = binding.split_once('=') {
                    let action = action.trim();
                    if matches!(action, "unbind" | "ignore") { continue; }
                    entries.push(Shortcut {
                        key: key.trim().replace('+', " + "),
                        action: action.replace('_', " ").replace(':', " → "),
                    });
                }
            }
        }
    }
    if entries.is_empty() {
        serde_json::from_str(include_str!("../shortcuts.json")).expect("valid bundled shortcuts")
    } else { entries }
}

fn listener(sender: mpsc::Sender<OverlayEvent>, delay: Duration) {
    thread::spawn(move || {
        let mut detector = input::DoubleControl::default();
        let mut codes = HashMap::new();
        let errors = sender.clone();
        let result = rdev::listen(move |event| {
            let (key, value) = match event.event_type {
                rdev::EventType::KeyPress(key) => (key, 1),
                rdev::EventType::KeyRelease(key) => (key, 0),
                _ => return,
            };
            let code = match key {
                rdev::Key::ControlLeft => evdev::Key::KEY_LEFTCTRL.code(),
                rdev::Key::ControlRight => evdev::Key::KEY_RIGHTCTRL.code(),
                other => {
                    let next = (codes.len() + 256) as u16;
                    *codes.entry(other).or_insert(next)
                }
            };
            if detector.event(code, value, Instant::now(), delay) {
                let _ = sender.send(OverlayEvent::DoubleTap);
            }
        });
        if let Err(error) = result {
            let _ = errors.send(OverlayEvent::Error(format!("Keyboard listener failed: {error:?}. Open from the tray instead.")));
        }
    });
}

fn toggle(window: &gtk::ApplicationWindow) {
    if window.is_visible() { window.set_visible(false); }
    else { window.present(); }
}

fn accept_tap(last: &mut Option<Instant>, now: Instant) -> bool {
    if last.is_some_and(|previous| now.duration_since(previous) < Duration::from_millis(120)) { return false; }
    *last = Some(now);
    true
}

fn main() -> glib::ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "--version") {
        println!("clavis {}", env!("CARGO_PKG_VERSION")); return glib::ExitCode::SUCCESS;
    }
    if args.iter().any(|arg| arg == "--list-keyboards") {
        for keyboard in input::candidates() {
            println!("{}\t{}", keyboard.path.display(), keyboard.name.as_deref().unwrap_or("Unnamed keyboard"));
        }
        return glib::ExitCode::SUCCESS;
    }
    if args.iter().any(|arg| arg == "--diagnose") {
        println!("Tap and release Ctrl twice within 380 ms. Ctrl+C to exit. All configured keyboards are monitored.");
        println!("Only connection notices and double-tap detections are printed; other keystrokes are not logged.");
        let (tx, rx) = mpsc::channel();
        if let Err(error) = input::start(tx, Duration::from_millis(380)) { eprintln!("{error}"); return glib::ExitCode::FAILURE; }
        for event in rx {
            match event {
                OverlayEvent::Toggle | OverlayEvent::DoubleTap => println!("Detected double Ctrl"),
                OverlayEvent::InputNotice(message) => println!("{message}"),
                OverlayEvent::Error(error) => { eprintln!("{error}"); return glib::ExitCode::FAILURE; }
                _ => {}
            }
        }
        return glib::ExitCode::FAILURE;
    }
    if args.iter().any(|arg| arg == "--check-input") {
        let keyboards = match input::configured() {
            Ok(keyboards) => keyboards,
            Err(error) => { eprintln!("{error}"); return glib::ExitCode::FAILURE; }
        };
        let mut readable = 0;
        for keyboard in &keyboards {
            match input::open(keyboard) {
                Ok((device, path)) => { readable += 1; println!("Readable: {} ({})", device.name().unwrap_or("keyboard"), path.display()); }
                Err(error) => eprintln!("{error}"),
            }
        }
        println!("{readable}/{} configured keyboards readable. Use --list-keyboards to find other devices.", keyboards.len());
        return if readable == keyboards.len() { glib::ExitCode::SUCCESS } else { glib::ExitCode::FAILURE };
    }
    let capture = args.windows(2).find(|pair| pair[0] == "--capture").map(|pair| pair[1].clone());
    let tool_name = args.windows(2).find(|pair| pair[0] == "--tool").map(|pair| pair[1].clone());
    let show = args.iter().any(|arg| arg == "--show") || capture.is_some();
    let delay = env::var("CLAVIS_TAP_DELAY").or_else(|_| env::var("GHOSTTY_EE_DELAY")).ok().and_then(|s| s.parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v > 0.0 && *v <= 5.0).unwrap_or(0.38);
    let app = gtk::Application::builder().application_id("dev.baul.Clavis").build();
    let first = Rc::new(Cell::new(true));
    app.connect_activate(move |app| {
        if !first.replace(false) {
            if let Some(window) = app.windows().first() { window.present(); }
            return;
        }
        let use_evdev = env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland")
            || env::var_os("CLAVIS_KEYBOARD").or_else(|| env::var_os("GHOSTTY_EE_KEYBOARD")).is_some();
        let hold = app.hold();
        app.connect_shutdown(move |_| { let _ = &hold; });
        let tools = Rc::new(catalog::tools());
        let ui::Ui { window, selector, status } = ui::build(app, tools.clone());
        if let Some(name) = &tool_name {
            if let Some(index) = tools.iter().position(|tool| tool.name.eq_ignore_ascii_case(name)) { selector.set_selected(index as u32); }
            else { eprintln!("Unknown tool: {name}"); }
        }
        let (tx, rx) = mpsc::channel();
        tray::start(tx.clone(), tools.iter().map(|tool| tool.name.clone()).collect());
        for (name, event) in [("toggle", OverlayEvent::Toggle), ("show", OverlayEvent::Show), ("quit", OverlayEvent::Quit)] {
            let action = gtk::gio::SimpleAction::new(name, None);
            let tx = tx.clone();
            action.connect_activate(move |_, _| {
                let message = match event { OverlayEvent::Toggle => OverlayEvent::Toggle, OverlayEvent::Show => OverlayEvent::Show, _ => OverlayEvent::Quit };
                let _ = tx.send(message);
            });
            app.add_action(&action);
        }
        if use_evdev {
            if !input::configured().is_ok_and(|keyboards| keyboards.iter().any(|keyboard| input::open(keyboard).is_ok())) {
                let _ = tx.send(OverlayEvent::Error("No configured keyboard is readable. Run clavis --list-keyboards and enable-keyboard.sh --all.".into()));
            }
            if let Err(error) = input::start(tx.clone(), Duration::from_secs_f64(delay)) {
                let _ = tx.send(OverlayEvent::Error(error));
            }
        } else { listener(tx, Duration::from_secs_f64(delay)); }
        let retained_window = window.clone(); let app_clone = app.clone();
        let mut last_keyboard_toggle = None;
        glib::timeout_add_local(Duration::from_millis(25), move || {
            for event in rx.try_iter() {
                match event {
                    OverlayEvent::Toggle | OverlayEvent::DoubleTap => {
                        if matches!(&event, OverlayEvent::DoubleTap) && !accept_tap(&mut last_keyboard_toggle, Instant::now()) { continue; }
                        eprintln!("Detected double Ctrl/toggle: {} overlay", if retained_window.is_visible() { "hiding" } else { "showing" });
                        toggle(&retained_window);
                    }
                    OverlayEvent::Show => retained_window.present(),
                    OverlayEvent::Hide => retained_window.set_visible(false),
                    OverlayEvent::SelectTool(index) => { selector.set_selected(index); retained_window.present(); }
                    OverlayEvent::Quit => { app_clone.quit(); return glib::ControlFlow::Break; }
                    OverlayEvent::InputNotice(message) => { eprintln!("{message}"); status.set_tooltip_text(Some(&message)); }
                    OverlayEvent::Error(error) => {
                        eprintln!("{error}"); status.set_text("Keyboard unavailable · open from tray");
                        status.set_tooltip_text(Some(&error)); retained_window.present();
                    }
                }
            }
            glib::ControlFlow::Continue
        });
        if show { window.present(); }
        if let Some(path) = capture.clone() {
            let window = window.clone(); let app = app.clone();
            glib::timeout_add_local_once(Duration::from_millis(1000), move || {
                match ui::capture(&window, &path) {
                    Ok(()) => println!("Captured {}×{} to {path}", window.width(), window.height()),
                    Err(error) => eprintln!("Capture failed: {error}"),
                }
                app.quit();
            });
        }
    });
    app.run_with_args(&["clavis"])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn composite_keyboard_events_are_deduplicated() {
        let now = Instant::now(); let mut last = None;
        assert!(accept_tap(&mut last, now));
        assert!(!accept_tap(&mut last, now + Duration::from_millis(20)));
        assert!(accept_tap(&mut last, now + Duration::from_millis(200)));
    }
    #[test]
    fn bundled_shortcuts_are_valid() {
        let entries: Vec<Shortcut> = serde_json::from_str(include_str!("../shortcuts.json")).unwrap();
        assert!(!entries.is_empty());
        assert!(entries.iter().all(|s| !s.key.is_empty() && !s.action.is_empty()));
    }
}
