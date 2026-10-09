use crate::OverlayEvent;
use std::{sync::mpsc::Sender, thread};

struct Tray {
    sender: Sender<OverlayEvent>,
    names: Vec<String>,
}

impl ksni::Tray for Tray {
    fn id(&self) -> String { "clavis".into() }
    fn title(&self) -> String { "Clavis · Terminal reference".into() }
    fn icon_name(&self) -> String { "clavis".into() }
    fn icon_pixmap(&self) -> Vec<ksni::Icon> { vec![icon()] }
    fn activate(&mut self, _: i32, _: i32) { let _ = self.sender.send(OverlayEvent::Toggle); }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::{StandardItem, SubMenu};
        let tools = self.names.iter().enumerate().map(|(index, name)| StandardItem {
            label: name.clone(),
            activate: Box::new(move |tray: &mut Self| { let _ = tray.sender.send(OverlayEvent::SelectTool(index as u32)); }),
            ..Default::default()
        }.into()).collect();
        vec![
            StandardItem { label: "Show reference".into(), activate: Box::new(|tray: &mut Self| { let _ = tray.sender.send(OverlayEvent::Show); }), ..Default::default() }.into(),
            StandardItem { label: "Hide reference".into(), activate: Box::new(|tray: &mut Self| { let _ = tray.sender.send(OverlayEvent::Hide); }), ..Default::default() }.into(),
            SubMenu { label: "Tools".into(), submenu: tools, ..Default::default() }.into(),
            ksni::MenuItem::Separator,
            StandardItem { label: "Quit".into(), icon_name: "application-exit".into(), activate: Box::new(|tray: &mut Self| { let _ = tray.sender.send(OverlayEvent::Quit); }), ..Default::default() }.into(),
        ]
    }
}

// Generated from the same SVG as the installed icon. ARGB network byte order.
fn icon() -> ksni::Icon {
    ksni::Icon {
        width: 32, height: 32,
        data: include_bytes!("../packaging/clavis-tray.argb").to_vec(),
    }
}

pub fn start(sender: Sender<OverlayEvent>, names: Vec<String>) {
    thread::spawn(move || {
        if let Err(error) = ksni::TrayService::new(Tray { sender, names }).run() {
            eprintln!("Tray unavailable: {error}. Use clavis --show to open and pkill -x clavis to quit.");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn icon_has_correct_pixel_layout() {
        let icon = icon(); assert_eq!((icon.width, icon.height), (32, 32));
        assert_eq!(icon.data.len(), 32 * 32 * 4);
        assert!(icon.data.chunks_exact(4).any(|pixel| pixel[0] == 255));
        assert_eq!(icon.data[0], 0, "Icon corners must remain transparent");
    }
}
