use evdev::{Device, InputEventKind, Key};
use std::{collections::HashSet, env, fs, path::{Path, PathBuf}, sync::mpsc::Sender, thread, time::{Duration, Instant}};

#[derive(Clone, Debug)]
pub struct Keyboard {
    pub path: PathBuf,
    pub name: Option<String>,
}

pub fn config_path() -> PathBuf {
    config_home().join("clavis/keyboards")
}

fn config_home() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".config"))
}

pub fn candidates() -> Vec<Keyboard> {
    let mut keyboards = Vec::new();
    if let Ok(entries) = fs::read_dir("/sys/class/input") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with("event") { continue; }
            let device = entry.path().join("device");
            let keys = fs::read_to_string(device.join("capabilities/key")).unwrap_or_default();
            if !bitmap_has(&keys, Key::KEY_LEFTCTRL.code()) && !bitmap_has(&keys, Key::KEY_RIGHTCTRL.code()) { continue; }
            let name = fs::read_to_string(device.join("name")).ok().map(|s| s.trim().to_string());
            keyboards.push(Keyboard { path: PathBuf::from("/dev/input").join(entry.file_name()), name });
        }
    }
    keyboards.sort_by(|a, b| a.path.cmp(&b.path));
    keyboards
}

fn bitmap_has(keys: &str, code: u16) -> bool {
    let bits = usize::BITS;
    keys.split_whitespace().rev().nth((u32::from(code) / bits) as usize)
        .and_then(|word| u64::from_str_radix(word, 16).ok())
        .is_some_and(|word| word & (1u64 << (u32::from(code) % bits)) != 0)
}

fn parse_config(text: &str) -> Vec<Keyboard> {
    let mut seen = HashSet::new();
    text.lines().filter_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { return None; }
        let (path, name) = line.split_once('\t').map(|(p, n)| (p, Some(n.to_string()))).unwrap_or((line, None));
        let path = PathBuf::from(path);
        let resolved = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if !seen.insert(resolved) { return None; }
        Some(Keyboard { path, name })
    }).collect()
}

pub fn configured() -> Result<Vec<Keyboard>, String> {
    if let Some(path) = env::var_os("CLAVIS_KEYBOARD").or_else(|| env::var_os("GHOSTTY_EE_KEYBOARD")) {
        return Ok(vec![Keyboard { path: PathBuf::from(path), name: None }]);
    }
    let text = fs::read_to_string(config_path())
        .or_else(|_| fs::read_to_string(config_home().join("ghostty-ee-overlay/keyboard")))
        .map_err(|_| "Choose your keyboards: run clavis --list-keyboards, then enable-keyboard.sh --all. Keyboard access requires explicit permission.".to_string())?;
    let keyboards = parse_config(&text);
    if keyboards.is_empty() { return Err("No keyboards configured. Run enable-keyboard.sh --all.".into()); }
    Ok(keyboards)
}

/// Isolated GTK tests and captures show synthetic status without reading any device data.
pub fn isolated_diagnostic(language: crate::locale::Language) -> String {
    language.text("isolatedDiagnostic").to_string()
}

/// Production diagnostics read Clavis' selection and sysfs name/capability metadata only;
/// they do not open /dev/input, inspect ACLs, or read keyboard events.
pub fn diagnostic_summary(language: crate::locale::Language) -> String {
    let setup = language.text("compositorGuide");
    let title = language.text("diagnosticTitle");
    match configured() {
        Ok(keyboards) => {
            let live = candidates();
            let devices = keyboards.iter().map(|keyboard| diagnostic_device(keyboard, &live, language)).collect::<Vec<_>>().join("\n");
            format!(
                "{title}:\n{devices}\n{} {}\n{}\n\n{setup}",
                language.text("devicePermissions"), language.text("checkInputHelp"),
                language.text("readOnlyDiag")
            )
        }
        Err(_) => format!("{title}:\n{}\n{}\n\n{setup}", language.text("doubleCtrlNotConfigured"), language.text("checkInputHelp")),
    }
}

fn diagnostic_device(configured: &Keyboard, live: &[Keyboard], language: crate::locale::Language) -> String {
    diagnostic_device_state(configured, sysfs_name(&configured.path).as_deref(), live, language)
}

fn diagnostic_device_state(configured: &Keyboard, current: Option<&str>, live: &[Keyboard], language: crate::locale::Language) -> String {
    let expected = configured.name.as_deref();
    let line = if let Some(expected_name) = expected {
        if current == Some(expected_name) {
            format!("{expected_name} — {} ({})", configured.path.display(), language.text("sysfsVisible"))
        } else if let Some(reconnected) = matching_named_device(expected_name, live) {
            format!("{expected_name} — {}: {}; {} {} {}", configured.path.display(), language.text("staleDevice"), language.text("candidateLabel"), reconnected.path.display(), language.text("candidateVisible"))
        } else {
            format!("{expected_name} — {} {}", configured.path.display(), language.text("notVisible"))
        }
    } else if let Some(name) = current {
        format!("{name} — {} ({})", configured.path.display(), language.text("sysfsVisible"))
    } else {
        format!("{} {}", configured.path.display(), language.text("notVisible"))
    };
    line
}

fn matching_named_device<'a>(name: &str, devices: &'a [Keyboard]) -> Option<&'a Keyboard> {
    devices.iter().find(|device| device.name.as_deref() == Some(name))
}

fn sysfs_name(path: &Path) -> Option<String> {
    let real = fs::canonicalize(path).ok()?;
    let event = real.file_name()?;
    fs::read_to_string(Path::new("/sys/class/input").join(event).join("device/name")).ok().map(|s| s.trim().to_string())
}

fn resolve(keyboard: &Keyboard) -> Result<PathBuf, String> {
    if let Some(name) = &keyboard.name {
        if sysfs_name(&keyboard.path).as_ref() == Some(name) { return Ok(keyboard.path.clone()); }
        // Bluetooth devices may get a different event number when they reconnect.
        if let Some(found) = matching_named_device(name, &candidates()) {
            return Ok(found.path.clone());
        }
        Err(format!("Keyboard disconnected: {name}"))
    } else { Ok(keyboard.path.clone()) }
}

pub fn open(keyboard: &Keyboard) -> Result<(Device, PathBuf), String> {
    let path = resolve(keyboard)?;
    let device = Device::open(&path).map_err(|error| format!(
        "Cannot read {} ({}): {error}. Run enable-keyboard.sh --all to grant access.", keyboard.name.as_deref().unwrap_or("keyboard"), path.display()))?;
    if !device.supported_keys().is_some_and(|keys| keys.contains(Key::KEY_LEFTCTRL) || keys.contains(Key::KEY_RIGHTCTRL)) {
        return Err(format!("{} is not a Ctrl-capable keyboard.", path.display()));
    }
    Ok((device, path))
}

#[derive(Default)]
pub(crate) struct DoubleControl {
    pressed: HashSet<u16>,
    candidate: Option<(u16, Instant)>,
    last: Option<Instant>,
}

impl DoubleControl {
    pub(crate) fn event(&mut self, code: u16, value: i32, now: Instant, delay: Duration) -> bool {
        if value == 0 {
            self.pressed.remove(&code);
            if self.candidate.is_some_and(|(key, _)| key == code) {
                let (_, down) = self.candidate.take().unwrap();
                if !self.pressed.is_empty() || now.duration_since(down) > delay {
                    self.last = None;
                    return false;
                }
                // Wait for release so Ctrl+C/V/etc. can cancel the candidate.
                if self.last.is_some_and(|last| now.duration_since(last) <= delay) {
                    self.last = None;
                    return true;
                }
                self.last = Some(now);
            }
            return false;
        }
        if value != 1 || !self.pressed.insert(code) { return false; }
        if (code == Key::KEY_LEFTCTRL.code() || code == Key::KEY_RIGHTCTRL.code()) && self.pressed.len() == 1 {
            self.candidate = Some((code, now));
        } else {
            self.candidate = None;
            self.last = None;
        }
        false
    }
}

pub fn start(sender: Sender<super::OverlayEvent>, delay: Duration) -> Result<(), String> {
    let keyboards = configured()?;
    let mut watched = HashSet::new();
    for keyboard in keyboards {
        // Avoid duplicate listeners when two aliases point at the same device.
        let identity = resolve(&keyboard).ok().and_then(|path| fs::canonicalize(path).ok()).unwrap_or_else(|| keyboard.path.clone());
        if !watched.insert(identity) { continue; }
        let sender = sender.clone();
        thread::spawn(move || {
            let mut previous_error = String::new();
            loop {
                match open(&keyboard) {
                    Ok((mut device, path)) => {
                        let _ = sender.send(super::OverlayEvent::InputNotice(format!("Listening: {} ({})", device.name().unwrap_or("keyboard"), path.display())));
                        previous_error.clear();
                        let mut detector = DoubleControl::default();
                        loop {
                            match device.fetch_events() {
                                Ok(events) => for event in events {
                                    if let InputEventKind::Key(key) = event.kind() {
                                        if detector.event(key.code(), event.value(), Instant::now(), delay)
                                            && sender.send(super::OverlayEvent::DoubleTap).is_err() { return; }
                                    }
                                },
                                Err(error) => {
                                    let _ = sender.send(super::OverlayEvent::InputNotice(format!("Keyboard disconnected: {error}. Waiting for reconnection.")));
                                    break;
                                }
                            }
                        }
                    }
                    Err(error) => {
                        if error != previous_error {
                            if sender.send(super::OverlayEvent::InputNotice(error.clone())).is_err() { return; }
                            previous_error = error;
                        }
                    }
                }
                thread::sleep(Duration::from_secs(2));
            }
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tap(d: &mut DoubleControl, key: Key, now: Instant) -> bool {
        let delay = Duration::from_millis(380);
        assert!(!d.event(key.code(), 1, now, delay));
        d.event(key.code(), 0, now + Duration::from_millis(20), delay)
    }
    #[test]
    fn requires_two_released_ctrl_taps() {
        let mut d = DoubleControl::default(); let t = Instant::now(); let delay = Duration::from_millis(380);
        assert!(!tap(&mut d, Key::KEY_LEFTCTRL, t));
        assert!(!d.event(Key::KEY_LEFTCTRL.code(), 1, t + Duration::from_millis(100), delay));
        assert!(!d.event(Key::KEY_LEFTCTRL.code(), 2, t + Duration::from_millis(110), delay));
        assert!(!d.event(Key::KEY_LEFTCTRL.code(), 1, t + Duration::from_millis(120), delay));
        assert!(d.event(Key::KEY_LEFTCTRL.code(), 0, t + Duration::from_millis(130), delay));
    }
    #[test]
    fn both_ctrl_keys_work() {
        let mut d = DoubleControl::default(); let t = Instant::now();
        assert!(!tap(&mut d, Key::KEY_RIGHTCTRL, t));
        assert!(tap(&mut d, Key::KEY_RIGHTCTRL, t + Duration::from_millis(100)));
        assert!(!tap(&mut d, Key::KEY_LEFTCTRL, t + Duration::from_millis(200)));
        assert!(tap(&mut d, Key::KEY_RIGHTCTRL, t + Duration::from_millis(300)));
    }
    #[test]
    fn slow_taps_and_long_holds_do_not_trigger() {
        let mut d = DoubleControl::default(); let t = Instant::now(); let delay = Duration::from_millis(380);
        assert!(!tap(&mut d, Key::KEY_LEFTCTRL, t));
        assert!(!tap(&mut d, Key::KEY_LEFTCTRL, t + Duration::from_secs(1)));
        assert!(!d.event(Key::KEY_LEFTCTRL.code(), 1, t + Duration::from_millis(1100), delay));
        assert!(!d.event(Key::KEY_LEFTCTRL.code(), 0, t + Duration::from_millis(1600), delay));
    }
    #[test]
    fn ctrl_chords_cancel_even_the_second_tap() {
        let mut d = DoubleControl::default(); let t = Instant::now(); let delay = Duration::from_millis(380);
        assert!(!tap(&mut d, Key::KEY_LEFTCTRL, t));
        assert!(!d.event(Key::KEY_LEFTCTRL.code(), 1, t + Duration::from_millis(100), delay));
        assert!(!d.event(Key::KEY_C.code(), 1, t + Duration::from_millis(110), delay));
        assert!(!d.event(Key::KEY_C.code(), 0, t + Duration::from_millis(120), delay));
        assert!(!d.event(Key::KEY_LEFTCTRL.code(), 0, t + Duration::from_millis(130), delay));
        assert!(!tap(&mut d, Key::KEY_LEFTCTRL, t + Duration::from_millis(200)));
    }
    #[test]
    fn ordinary_ee_and_other_held_keys_do_not_trigger() {
        let mut d = DoubleControl::default(); let t = Instant::now(); let delay = Duration::from_millis(380);
        assert!(!tap(&mut d, Key::KEY_E, t));
        assert!(!tap(&mut d, Key::KEY_E, t + Duration::from_millis(100)));
        d.event(Key::KEY_A.code(), 1, t + Duration::from_millis(200), delay);
        assert!(!tap(&mut d, Key::KEY_LEFTCTRL, t + Duration::from_millis(210)));
        assert!(!tap(&mut d, Key::KEY_LEFTCTRL, t + Duration::from_millis(310)));
    }
    #[test]
    fn detects_ctrl_bits_across_bitmap_words() {
        let left = format!("{:x}", 1u64 << Key::KEY_LEFTCTRL.code());
        assert!(bitmap_has(&left, Key::KEY_LEFTCTRL.code()));
        assert!(!bitmap_has(&left, Key::KEY_RIGHTCTRL.code()));
        let word = u32::from(Key::KEY_RIGHTCTRL.code()) / usize::BITS;
        let right = format!("{:x}{}", 1u64 << (u32::from(Key::KEY_RIGHTCTRL.code()) % usize::BITS), " 0".repeat(word as usize));
        assert!(bitmap_has(&right, Key::KEY_RIGHTCTRL.code()));
    }
    #[test]
    fn supports_named_multiple_keyboards_and_legacy_paths() {
        let specs = parse_config("/dev/input/event3\tLaptop\n/dev/input/event18\tUSB Keyboard\n/dev/input/event18\tUSB Keyboard\n");
        assert_eq!(specs.len(), 2); assert_eq!(specs[1].name.as_deref(), Some("USB Keyboard"));
        assert_eq!(parse_config("/dev/input/event3\n").len(), 1);
        assert!(parse_config("# no devices\n").is_empty());
    }

    #[test]
    fn synthetic_reconnection_finds_renumbered_named_device_without_opening_it() {
        let configured = Keyboard { path: PathBuf::from("/dev/input/event3"), name: Some("USB Keyboard".into()) };
        let candidates = [
            Keyboard { path: PathBuf::from("/dev/input/event18"), name: Some("USB Keyboard".into()) },
            Keyboard { path: PathBuf::from("/dev/input/event19"), name: Some("Mouse".into()) },
        ];
        assert_eq!(matching_named_device("USB Keyboard", &candidates).unwrap().path, PathBuf::from("/dev/input/event18"));
        assert!(diagnostic_device_state(&configured, None, &candidates, crate::locale::Language::English).contains("candidate /dev/input/event18"));
        assert!(diagnostic_device_state(&configured, Some("USB Keyboard"), &candidates, crate::locale::Language::English).contains("permission not checked"));
        assert!(diagnostic_device_state(&configured, None, &candidates, crate::locale::Language::Spanish).contains("candidato /dev/input/event18"));
        assert!(matching_named_device("Absent", &candidates).is_none());
    }
}
