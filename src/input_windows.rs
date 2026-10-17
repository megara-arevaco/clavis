use std::{
    collections::{HashMap, HashSet},
    env,
    path::PathBuf,
    sync::mpsc::Sender,
    thread,
    time::{Duration, Instant},
};

pub fn config_path() -> PathBuf {
    let base = env::var_os("CLAVIS_TEST_CONFIG_HOME")
        .or_else(|| env::var_os("APPDATA"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env::var_os("USERPROFILE").unwrap_or_default()).join("AppData/Roaming"));
    base.join("Clavis/keyboards")
}

pub fn isolated_diagnostic(language: crate::locale::Language) -> String {
    language.text("isolatedDiagnostic").to_string()
}

pub fn diagnostic_summary(language: crate::locale::Language) -> String {
    language.text("windowsKeyboardListener").to_string()
}

pub fn start(sender: Sender<super::OverlayEvent>, delay: Duration) -> Result<(), String> {
    thread::spawn(move || {
        let mut detector = DoubleControl::default();
        let mut codes = HashMap::new();
        let errors = sender.clone();
        let result = rdev::listen(move |event| {
            let (key, value) = match event.event_type {
                rdev::EventType::KeyPress(key) => (key, 1),
                rdev::EventType::KeyRelease(key) => (key, 0),
                _ => return,
            };
            let code = match key {
                rdev::Key::ControlLeft => 29,
                rdev::Key::ControlRight => 97,
                other => {
                    let next = (codes.len() + 256) as u16;
                    *codes.entry(other).or_insert(next)
                }
            };
            if detector.event(code, value, Instant::now(), delay) {
                let _ = sender.send(super::OverlayEvent::DoubleTap);
            }
        });
        if let Err(error) = result {
            let _ = errors.send(super::OverlayEvent::Error(format!(
                "Windows keyboard listener failed: {error:?}. Open Clavis from the system tray."
            )));
        }
    });
    Ok(())
}

#[derive(Default)]
struct DoubleControl {
    pressed: HashSet<u16>,
    candidate: Option<(u16, Instant)>,
    last: Option<Instant>,
}

impl DoubleControl {
    fn event(&mut self, code: u16, value: i32, now: Instant, delay: Duration) -> bool {
        if value == 0 {
            self.pressed.remove(&code);
            if self.candidate.is_some_and(|(key, _)| key == code) {
                let (_, down) = self.candidate.take().unwrap();
                if !self.pressed.is_empty() || now.duration_since(down) > delay {
                    self.last = None;
                    return false;
                }
                if self.last.is_some_and(|last| now.duration_since(last) <= delay) {
                    self.last = None;
                    return true;
                }
                self.last = Some(now);
            }
            return false;
        }
        if value != 1 || !self.pressed.insert(code) { return false; }
        if (code == 29 || code == 97) && self.pressed.len() == 1 {
            self.candidate = Some((code, now));
        } else {
            self.candidate = None;
            self.last = None;
        }
        false
    }
}

