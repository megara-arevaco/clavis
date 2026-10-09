use crate::Shortcut;
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Deserialize)]
pub struct Tool {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub note: String,
    pub entries: Vec<Shortcut>,
}

pub fn tools() -> Vec<Tool> {
    let mut tools = vec![Tool {
        name: "Ghostty".into(), kind: "Keys".into(),
        source: "https://ghostty.org/docs/config/keybind".into(),
        note: "Linux defaults; explicit config bindings replace this reference, not a full effective keymap.".into(),
        entries: crate::shortcuts(),
    }];
    tools.extend(serde_json::from_str::<Vec<Tool>>(include_str!("../catalog.json")).expect("valid bundled catalog"));
    tools
}

fn preference_path() -> PathBuf {
    crate::input::config_path().with_file_name("last-tool")
}

pub fn selected_tool(tools: &[Tool]) -> u32 {
    let name = fs::read_to_string(preference_path()).or_else(|_| {
        let legacy = preference_path().parent().unwrap().parent().unwrap().join("ghostty-ee-overlay/last-tool");
        fs::read_to_string(legacy)
    }).unwrap_or_default();
    tools.iter().position(|tool| tool.name == name.trim()).unwrap_or(0) as u32
}

pub fn remember_tool(name: &str) {
    let path = preference_path();
    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_ok() {
            if let Err(error) = fs::write(path, name) { eprintln!("Cannot save selected tool: {error}"); }
        }
    }
}

pub fn matches(entry: &Shortcut, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    entry.key.to_lowercase().contains(&query) || entry.action.to_lowercase().contains(&query)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ten_references_have_sources_and_content() {
        let tools: Vec<Tool> = serde_json::from_str(include_str!("../catalog.json")).unwrap();
        assert_eq!(tools.len(), 10);
        let mut names = std::collections::HashSet::new();
        for tool in tools {
            assert!(names.insert(tool.name));
            assert!(tool.source.starts_with("https://"));
            assert!(tool.entries.len() >= 8);
            assert!(tool.entries.iter().all(|e| !e.key.is_empty() && !e.action.is_empty()));
        }
    }
    #[test]
    fn search_matches_commands_and_descriptions() {
        let entry = Shortcut { key: "git status -sb".into(), action: "Branch and working tree".into() };
        assert!(matches(&entry, "STATUS"));
        assert!(matches(&entry, "branch"));
        assert!(matches(&entry, ""));
        assert!(!matches(&entry, "banana"));
    }
}
