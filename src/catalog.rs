use crate::Shortcut;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::{Path, PathBuf}};

const PERSONAL_SCHEMA_VERSION: u32 = 1;
const MAX_CATALOG_BYTES: usize = 1_048_576;
const NOT_RECORDED: &str = "Not recorded";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    pub name: String,
    pub kind: String,
    pub source: String,
    pub note: String,
    #[serde(default = "default_origin")]
    pub origin: String,
    #[serde(default = "not_recorded")]
    pub version: String,
    #[serde(default = "not_recorded")]
    pub reviewed: String,
    #[serde(default)]
    pub personal: bool,
    pub entries: Vec<Shortcut>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersonalCatalog {
    schema_version: u32,
    references: Vec<Tool>,
}

fn default_origin() -> String { "Bundled reference".into() }
fn not_recorded() -> String { NOT_RECORDED.into() }

fn archived_version(tool: &str) -> &'static str {
    match tool {
        "curl" => "curl 8.23.0 man page snapshot",
        "tmux" => "tmux.1 revision 1.1181 (2026-10-08)",
        "Vim" => "Vim 9.2 reference (last change 2026-05-17)",
        _ => NOT_RECORDED,
    }
}

pub fn tools() -> Vec<Tool> {
    let mut ghostty_entries = crate::shortcuts();
    let ghostty_custom = ghostty_entries.iter().any(|entry| entry.origin.starts_with("User Ghostty config"));
    let mut tools = vec![Tool {
        name: "Ghostty".into(), kind: "Keys".into(),
        source: "https://ghostty.org/docs/config/keybind".into(),
        note: if ghostty_custom {
            "Explicit keybindings from your Ghostty config. Includes, resets, and conditional bindings are not resolved.".into()
        } else {
            "Verified Linux defaults. Includes, resets, and conditional bindings in Ghostty config are not resolved.".into()
        },
        origin: if ghostty_custom { "User Ghostty config (explicit bindings)" } else { "Ghostty Linux defaults" }.into(),
        version: NOT_RECORDED.into(), reviewed: NOT_RECORDED.into(), personal: false, entries: Vec::new(),
    }];
    stamp_entries(&mut ghostty_entries, "https://ghostty.org/docs/config/keybind", NOT_RECORDED, NOT_RECORDED);
    for entry in &mut ghostty_entries {
        if let Some(warning) = curated_warning("Ghostty", &entry.key) { entry.warning = warning.into(); }
    }
    tools[0].entries = ghostty_entries;

    let mut bundled: Vec<Tool> = serde_json::from_str(include_str!("../catalog.json")).expect("valid bundled catalog");
    for tool in &mut bundled {
        if tool.origin == "Bundled reference" { tool.origin = "Curated upstream reference".into(); }
        if tool.version == NOT_RECORDED { tool.version = archived_version(&tool.name).into(); }
        stamp_entries(&mut tool.entries, &tool.source.clone(), &tool.version.clone(), &tool.reviewed.clone());
        for entry in &mut tool.entries {
            if entry.warning.is_empty() {
                if let Some(warning) = curated_warning(&tool.name, &entry.key) { entry.warning = warning.into(); }
            }
        }
    }
    tools.extend(bundled);

    let path = personal_catalog_path();
    if fs::symlink_metadata(&path).is_ok() {
        match read_personal_file(&path).and_then(|contents| parse_personal_catalog(&contents, &tools)) {
            Ok(mut personal) => {
                for tool in &mut personal { tool.personal = true; }
                tools.extend(personal);
            },
            Err(error) => eprintln!("Ignoring invalid personal catalog {}: {error}", path.display()),
        }
    }
    tools
}

fn stamp_entries(entries: &mut [Shortcut], origin: &str, version: &str, reviewed: &str) {
    for entry in entries {
        if entry.origin.is_empty() { entry.origin = origin.into(); }
        if entry.version.is_empty() || entry.version == NOT_RECORDED { entry.version = version.into(); }
        if entry.reviewed.is_empty() || entry.reviewed == NOT_RECORDED { entry.reviewed = reviewed.into(); }
    }
}

pub fn personal_catalog_path() -> PathBuf {
    crate::input::config_path().with_file_name("personal-catalog.json")
}

fn preference_path(name: &str) -> PathBuf {
    crate::input::config_path().with_file_name(name)
}

pub fn selected_tool(tools: &[Tool]) -> u32 {
    let name = fs::read_to_string(preference_path("last-tool")).or_else(|_| {
        let legacy = preference_path("last-tool").parent().unwrap().parent().unwrap().join("ghostty-ee-overlay/last-tool");
        fs::read_to_string(legacy)
    }).unwrap_or_default();
    tools.iter().position(|tool| tool.name == name.trim()).unwrap_or(0) as u32
}

pub fn remember_tool(name: &str) -> Result<(), String> {
    save_atomic(&preference_path("last-tool"), name.as_bytes())
}

pub fn matches(entry: &Shortcut, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    entry.key.to_lowercase().contains(&query) || entry.action.to_lowercase().contains(&query)
}

pub fn favorite_id(tool: &str, key: &str) -> String {
    format!("{tool}\u{1f}{key}")
}

pub fn load_favorites() -> HashSet<String> {
    let Ok(contents) = fs::read_to_string(preference_path("favorites.json")) else { return HashSet::new(); };
    serde_json::from_str(&contents).unwrap_or_default()
}

pub fn save_favorites(favorites: &HashSet<String>) -> Result<(), String> {
    let path = preference_path("favorites.json");
    let contents = serde_json::to_string_pretty(favorites).map_err(|error| error.to_string())?;
    save_atomic(&path, contents.as_bytes())
}

pub fn parse_personal_catalog(contents: &str, existing: &[Tool]) -> Result<Vec<Tool>, String> {
    if contents.len() > MAX_CATALOG_BYTES { return Err("Catalog exceeds the 1 MiB size limit.".into()); }
    let catalog: PersonalCatalog = serde_json::from_str(contents).map_err(|error| format!("Invalid JSON: {error}"))?;
    if catalog.schema_version != PERSONAL_SCHEMA_VERSION {
        return Err(format!("Unsupported schema_version {}; expected {PERSONAL_SCHEMA_VERSION}.", catalog.schema_version));
    }
    if catalog.references.len() > 50 {
        return Err("A personal catalog may contain at most 50 references.".into());
    }
    let mut names: HashSet<String> = existing.iter().map(|tool| tool.name.to_lowercase()).collect();
    for tool in &catalog.references {
        if tool.name.trim().is_empty() || tool.name.len() > 60 || !valid_text(&tool.name) {
            return Err("Reference names must be 1–60 characters and contain no control characters.".into());
        }
        if !names.insert(tool.name.to_lowercase()) {
            return Err(format!("Reference name already exists: {}. Imports never overwrite existing content.", tool.name));
        }
        if !matches!(tool.kind.as_str(), "Commands" | "Keys") {
            return Err(format!("{}: kind must be Commands or Keys.", tool.name));
        }
        if !valid_https_url(&tool.source) {
            return Err(format!("{}: source must be an HTTPS documentation URL.", tool.name));
        }
        if tool.note.len() > 500 || !valid_text(&tool.note) || tool.origin.trim().is_empty()
            || tool.origin.len() > 160 || !valid_text(&tool.origin) {
            return Err(format!("{}: invalid note or origin metadata.", tool.name));
        }
        if !valid_metadata(&tool.version) || !valid_reviewed(&tool.reviewed) {
            return Err(format!("{}: version and reviewed metadata must be present and at most 80 characters.", tool.name));
        }
        if tool.origin == "Bundled reference" {
            return Err(format!("{}: origin metadata is required.", tool.name));
        }
        if tool.entries.is_empty() || tool.entries.len() > 500 {
            return Err(format!("{}: include between 1 and 500 references.", tool.name));
        }
        for entry in &tool.entries {
            if entry.key.trim().is_empty() || entry.key.len() > 256 || !valid_text(&entry.key)
                || entry.action.trim().is_empty() || entry.action.len() > 500 || !valid_text(&entry.action) {
                return Err(format!("{}: reference keys/actions must be non-empty and within their size limits.", tool.name));
            }
            if entry.origin.trim().is_empty() || entry.origin.len() > 256 || !valid_text(&entry.origin)
                || !valid_metadata(&entry.version) || !valid_reviewed(&entry.reviewed)
                || entry.warning.len() > 300 || !valid_text(&entry.warning) {
                return Err(format!("{}: every entry needs valid origin, version and reviewed metadata.", tool.name));
            }
        }
    }
    Ok(catalog.references)
}

pub fn validate_personal_tool(tool: Tool, existing: &[Tool]) -> Result<Tool, String> {
    let contents = serde_json::to_string(&PersonalCatalog { schema_version: PERSONAL_SCHEMA_VERSION, references: vec![tool] })
        .map_err(|error| format!("Cannot validate personal reference: {error}"))?;
    parse_personal_catalog(&contents, existing)?.into_iter().next()
        .ok_or_else(|| "Personal reference is empty.".into())
}

pub fn import_personal_contents(contents: &str, tools: &mut Vec<Tool>) -> Result<usize, String> {
    let mut imported = parse_personal_catalog(contents, tools)?;
    let count = imported.len();
    for tool in &mut imported { tool.personal = true; }
    let mut combined = tools.clone();
    combined.extend(imported);
    let output = export_personal_catalog(&combined)?;
    save_atomic(&personal_catalog_path(), output.as_bytes())?;
    *tools = combined;
    Ok(count)
}

fn valid_https_url(value: &str) -> bool {
    let Some(host_and_path) = value.strip_prefix("https://") else { return false; };
    let host = host_and_path.split('/').next().unwrap_or("");
    !host.is_empty() && value.len() <= 512 && valid_text(value) && !host.contains('@')
}

fn valid_metadata(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 80 && valid_text(value)
}

fn valid_reviewed(value: &str) -> bool {
    if value == NOT_RECORDED { return true; }
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-'
        || !bytes.iter().enumerate().all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit()) {
        return false;
    }
    let year = value[0..4].parse::<u32>().unwrap_or(0);
    let month = value[5..7].parse::<u32>().unwrap_or(0);
    let day = value[8..10].parse::<u32>().unwrap_or(0);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month { 1 | 3 | 5 | 7 | 8 | 10 | 12 => 31, 4 | 6 | 9 | 11 => 30, 2 if leap => 29, 2 => 28, _ => 0 };
    day >= 1 && day <= days
}

fn valid_text(value: &str) -> bool {
    !value.chars().any(char::is_control)
}

pub fn export_personal_catalog(tools: &[Tool]) -> Result<String, String> {
    let references: Vec<Tool> = tools.iter().filter(|tool| tool.personal).cloned().collect();
    if references.len() > 50 { return Err("A personal catalog may contain at most 50 references.".into()); }
    let value = PersonalCatalog { schema_version: PERSONAL_SCHEMA_VERSION, references };
    let contents = serde_json::to_string_pretty(&value).map_err(|error| error.to_string())?;
    if contents.len() > MAX_CATALOG_BYTES { return Err("Catalog exceeds the 1 MiB size limit.".into()); }
    Ok(contents)
}

pub fn save_atomic(path: &Path, contents: &[u8]) -> Result<(), String> {
    write_atomic(path, contents, true)
}

/// Create a new export without replacing an existing destination, including a symlink.
pub fn save_new(path: &Path, contents: &[u8]) -> Result<(), String> {
    write_atomic(path, contents, false)
}

pub fn read_personal_file(path: &Path) -> Result<String, String> {
    reject_symlink_components(path)?;
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("Cannot inspect import: {error}"))?;
    if !metadata.is_file() { return Err("Import must be a regular local file, not a directory or symlink.".into()); }
    if metadata.len() > MAX_CATALOG_BYTES as u64 { return Err("Catalog exceeds the 1 MiB size limit.".into()); }
    use std::io::Read;
    let file = fs::File::open(path).map_err(|error| format!("Cannot open import: {error}"))?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take((MAX_CATALOG_BYTES + 1) as u64).read_to_end(&mut bytes)
        .map_err(|error| format!("Cannot read import: {error}"))?;
    if bytes.len() > MAX_CATALOG_BYTES { return Err("Catalog exceeds the 1 MiB size limit.".into()); }
    String::from_utf8(bytes).map_err(|error| format!("Import is not UTF-8 JSON: {error}"))
}

fn write_atomic(path: &Path, contents: &[u8], replace: bool) -> Result<(), String> {
    let (parent, temp, file) = create_temporary(path, contents)?;
    if let Err(error) = file.sync_all() {
        drop(file);
        let _ = fs::remove_file(&temp);
        return Err(format!("Cannot sync temporary file: {error}"));
    }
    drop(file);
    let result = if replace {
        reject_symlink_components(path).and_then(|()| {
            if let Ok(metadata) = fs::symlink_metadata(path) {
                if !metadata.is_file() { return Err("Refusing to replace a non-regular file.".into()); }
            }
            #[cfg(windows)]
            { move_windows_file(&temp, path, true).map_err(|error| format!("Cannot install file: {error}")) }
            #[cfg(not(windows))]
            { fs::rename(&temp, path).map_err(|error| format!("Cannot install file: {error}")) }
        })
    } else {
        #[cfg(windows)]
        { move_windows_file(&temp, path, false).map_err(|error| format!("Cannot create file without replacing an existing destination: {error}")) }
        #[cfg(not(windows))]
        {
        fs::hard_link(&temp, path)
            .map_err(|error| format!("Cannot create file without replacing an existing destination: {error}"))
            .and_then(|()| fs::remove_file(&temp).map_err(|error| format!("Cannot remove temporary file: {error}")))
        }
    };
    if result.is_err() { let _ = fs::remove_file(&temp); }
    result?;
    if let Ok(directory) = fs::File::open(parent) { let _ = directory.sync_all(); }
    Ok(())
}

#[cfg(windows)]
fn move_windows_file(source: &Path, destination: &Path, replace: bool) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH};

    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination.as_os_str().encode_wide().chain(Some(0)).collect();
    let flags = MOVEFILE_WRITE_THROUGH | if replace { MOVEFILE_REPLACE_EXISTING } else { 0 };
    let result = unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), flags) };
    if result == 0 { Err(std::io::Error::last_os_error()) } else { Ok(()) }
}

fn create_temporary(path: &Path, contents: &[u8]) -> Result<(PathBuf, PathBuf, fs::File), String> {
    let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty())
        .ok_or("Destination has no parent directory")?.to_path_buf();
    reject_symlink_components(&parent)?;
    fs::create_dir_all(&parent).map_err(|error| format!("Cannot create destination directory: {error}"))?;
    reject_symlink_components(&parent)?;
    reject_symlink_components(path)?;
    let name = path.file_name().ok_or("Destination has no file name")?.to_string_lossy();
    for attempt in 0..32u32 {
        let temp = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), attempt));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&temp) {
            Ok(mut file) => {
                use std::io::Write;
                if let Err(error) = file.write_all(contents) {
                    let _ = fs::remove_file(&temp);
                    return Err(format!("Cannot write temporary file: {error}"));
                }
                return Ok((parent, temp, file));
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("Cannot create temporary file: {error}")),
        }
    }
    Err("Cannot allocate a unique temporary file.".into())
}

pub(crate) fn reject_symlink_components(path: &Path) -> Result<(), String> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("Refusing to access a path through symlink {}.", current.display()));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("Cannot inspect path {}: {error}", current.display())),
        }
    }
    Ok(())
}

/// Curated warnings for bundled entries; an exact-key match is intentional, not inferred from shell syntax.
pub fn curated_warning(tool: &str, key: &str) -> Option<&'static str> {
    match (tool, key) {
        ("Git", "git switch <branch>") => Some("Changes the checked-out branch; confirm uncommitted work is safe to carry over."),
        ("Git", "git switch -c <branch>") => Some("Creates and checks out a branch; confirm the starting point."),
        ("Git", "git add <file>") => Some("Stages the selected file for a later commit; review the staged diff."),
        ("Git", "git commit -m '<message>'") => Some("Creates a commit from the staged changes; review them first."),
        ("Git", "git stash push") => Some("Moves tracked working changes into a stash; verify what will be saved."),
        ("Git", "git fetch") => Some("Contacts the remote and updates remote-tracking refs; it does not update the working tree."),
        ("SSH", "ssh <user>@<host>") => Some("Connects to a remote host; verify its host key before trusting the session."),
        ("SSH", "ssh -p <port> <host>") => Some("Connects to a remote host on a non-default port; verify its host key before trusting the session."),
        ("SSH", "ssh -i <key> <host>") => Some("Uses the selected identity file; keep private-key material confidential."),
        ("SSH", "ssh -v <host>") => Some("Verbose diagnostics can reveal hostnames and connection details; review before sharing."),
        ("SSH", "ssh -G <host>") => Some("Prints resolved SSH configuration, which may reveal internal hostnames, users or identity paths."),
        ("SSH", "ssh -L 8080:localhost:80 <host>") => Some("Forwards a local port to a remote service; confirm the destination and who can reach the local port."),
        ("SSH", "ssh -N -D 1080 <host>") => Some("Creates a SOCKS proxy that can route application traffic through the remote host."),
        ("curl", "curl -v <url>") => Some("Verbose output may reveal request headers or credentials; review before sharing."),
        ("curl", "curl -o <file> <url>") => Some("Writes to the named local file and may replace its contents."),
        ("curl", "curl --json @data.json <url>") => Some("Sends a POST request; the remote service may create or change data."),
        ("Vim", "dd") => Some("Deletes the current line from the buffer; use Vim undo if needed."),
        ("Vim", ":w / :q / :wq") => Some("The write forms save the buffer to disk and may replace file contents."),
        ("fzf", "Ctrl+R") => Some("Searches shell history through fzf integration; treat displayed history as private."),
        ("fzf", "fzf --preview 'head -80 {}'") => Some("Runs a preview command and displays file contents; avoid exposing private files."),
        ("ripgrep", "rg --hidden '<pattern>'") => Some("Includes hidden files that may contain private configuration or credentials."),
        ("fd", "fd -H '<pattern>'") => Some("Includes hidden paths that may contain private configuration or credentials."),
        ("fd", "fd -I '<pattern>'") => Some("Includes ignored paths that may contain generated or private files."),
        ("tmux", "tmux attach") => Some("Attaches to an existing session; its contents may be visible and input affects that session."),
        ("htop", "F9 / k") => Some("Sends a signal to the selected process; this can terminate it."),
        ("Ghostty", "Ctrl+Shift+W") => Some("Closes the current terminal tab and its foreground session may end."),
        ("Ghostty", "Ctrl+Shift+V") => Some("Pastes clipboard contents into the terminal; inspect it before pressing Enter."),
        _ => None,
    }
}

pub fn matches_sensitive(entry: &Shortcut) -> Option<String> {
    if !entry.warning.trim().is_empty() { return Some(entry.warning.clone()); }
    (entry.key.contains('<') && entry.key.contains('>'))
        .then(|| "Replace the highlighted placeholder before using this reference.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Shortcut;

    fn sample_personal() -> String {
        r#"{
          "schema_version": 1,
          "references": [{
            "name": "My Tool", "kind": "Commands", "source": "https://example.org/docs",
            "note": "Personal command reference", "origin": "My notes", "version": "2.1",
            "reviewed": "2026-10-10", "entries": [{
              "key": "my-tool <target>", "action": "Run against a selected target", "origin": "My notes",
              "version": "2.1", "reviewed": "2026-10-10"
            }]
          }]
        }"#.into()
    }

    fn fingerprint(bytes: &[u8]) -> u64 {
        bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3))
    }

    #[test]
    fn bundled_snapshot_fingerprint_covers_all_reviewed_entries() {
        let catalog = include_str!("../catalog.json");
        let tools: Vec<Tool> = serde_json::from_str(catalog).unwrap();
        assert_eq!(tools.iter().map(|tool| tool.entries.len()).sum::<usize>(), 94);
        assert_eq!(fingerprint(catalog.as_bytes()), 0xc664484e37598ed4, "catalog content changed: re-review every entry and update the coverage fingerprint");
        let ghostty = include_str!("../shortcuts.json");
        let entries: Vec<Shortcut> = serde_json::from_str(ghostty).unwrap();
        assert_eq!(entries.len(), 15);
        assert_eq!(fingerprint(ghostty.as_bytes()), 0x9ad90568bd0a2431, "Ghostty references changed: re-review every entry and update the coverage fingerprint");
    }

    #[test]
    fn every_bundled_entry_has_an_explicit_risk_classification() {
        let inventory: serde_json::Value = serde_json::from_str(include_str!("../research/catalog-risk-review.json")).unwrap();
        assert_eq!(inventory["schema_version"], 1);
        let catalog_text = include_str!("../catalog.json");
        let catalog: Vec<Tool> = serde_json::from_str(catalog_text).unwrap();
        let ghostty_text = include_str!("../shortcuts.json");
        let ghostty: Vec<Shortcut> = serde_json::from_str(ghostty_text).unwrap();
        let catalog_hash = format!("{:016x}", fingerprint(catalog_text.as_bytes()));
        let ghostty_hash = format!("{:016x}", fingerprint(ghostty_text.as_bytes()));
        assert_eq!(inventory["snapshots"]["catalog.json"].as_str(), Some(catalog_hash.as_str()));
        assert_eq!(inventory["snapshots"]["shortcuts.json"].as_str(), Some(ghostty_hash.as_str()));
        let mut classified_count = 0usize;
        let mut warning_count = 0usize;
        let mut check_tool = |name: &str, entries: &[Shortcut]| {
            let record = &inventory["tools"][name];
            let no_warning: HashSet<&str> = record["no_warning"].as_array().unwrap().iter().map(|key| key.as_str().unwrap()).collect();
            let warnings = record["warnings"].as_object().unwrap();
            let source_keys: HashSet<&str> = entries.iter().map(|entry| entry.key.as_str()).collect();
            let warning_keys: HashSet<&str> = warnings.keys().map(String::as_str).collect();
            assert_eq!(no_warning.len() + warning_keys.len(), entries.len(), "duplicate or missing classification for {name}");
            assert_eq!(no_warning.union(&warning_keys).copied().collect::<HashSet<_>>(), source_keys, "inventory key drift for {name}");
            for entry in entries {
                if let Some(warning) = warnings.get(&entry.key) {
                    assert_eq!(curated_warning(name, &entry.key), warning.as_str(), "warning text drift for {name}: {}", entry.key);
                    warning_count += 1;
                } else {
                    assert!(no_warning.contains(entry.key.as_str()));
                    assert!(curated_warning(name, &entry.key).is_none(), "unregistered warning for {name}: {}", entry.key);
                }
                classified_count += 1;
            }
        };
        for tool in &catalog { check_tool(&tool.name, &tool.entries); }
        check_tool("Ghostty", &ghostty);
        assert_eq!(classified_count, 109);
        assert_eq!(warning_count, 27);
        assert_eq!(inventory["tools"].as_object().unwrap().len(), 11);
    }

    #[test]
    fn archived_versions_match_the_included_upstream_snapshots() {
        assert!(include_str!("../research/curl.txt").contains("This man page describes curl 8.23.0."));
        assert!(include_str!("../research/tmux.txt").starts_with(".\\\" $OpenBSD: tmux.1,v 1.1181 2026/10/08"));
        assert!(include_str!("../research/vim.txt").starts_with("*quickref.txt*\tFor Vim version 9.2.  Last change: 2026 May 17"));
        assert_eq!(archived_version("curl"), "curl 8.23.0 man page snapshot");
        assert_eq!(archived_version("tmux"), "tmux.1 revision 1.1181 (2026-10-08)");
        assert_eq!(archived_version("Vim"), "Vim 9.2 reference (last change 2026-05-17)");
        assert_eq!(archived_version("Git"), "Not recorded");
    }

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
        let entry = Shortcut { key: "git status -sb".into(), action: "Branch and working tree".into(), origin: String::new(), version: String::new(), reviewed: String::new(), warning: String::new() };
        assert!(matches(&entry, "STATUS"));
        assert!(matches(&entry, "branch"));
        assert!(matches(&entry, ""));
        assert!(!matches(&entry, "banana"));
    }

    #[test]
    fn personal_catalog_import_is_validated_and_never_overwrites() {
        let imported = parse_personal_catalog(&sample_personal(), &[]).unwrap();
        assert_eq!(imported[0].entries[0].origin, "My notes");
        assert!(parse_personal_catalog(&sample_personal(), &imported).unwrap_err().contains("never overwrite"));
        let malformed = sample_personal().replace("\"schema_version\": 1", "\"schema_version\": 2");
        assert!(parse_personal_catalog(&malformed, &[]).unwrap_err().contains("Unsupported schema_version"));
        let malformed = sample_personal().replace("https://example.org/docs", "file:///etc/passwd");
        assert!(parse_personal_catalog(&malformed, &[]).unwrap_err().contains("HTTPS"));
        let malformed = sample_personal().replace("https://example.org/docs", "https://");
        assert!(parse_personal_catalog(&malformed, &[]).unwrap_err().contains("HTTPS"));
        let missing_origin = sample_personal().replace(", \"origin\": \"My notes\"", "");
        assert!(parse_personal_catalog(&missing_origin, &[]).unwrap_err().contains("origin metadata"));
        let malformed = sample_personal().replace("my-tool <target>", "");
        assert!(parse_personal_catalog(&malformed, &[]).unwrap_err().contains("keys/actions"));
        let malformed = sample_personal().replace("2026-10-10", "2026-02-30");
        assert!(parse_personal_catalog(&malformed, &[]).unwrap_err().contains("version and reviewed metadata"));
    }

    #[test]
    fn personal_export_roundtrips_only_personal_references() {
        let mut personal = parse_personal_catalog(&sample_personal(), &[]).unwrap();
        personal[0].personal = true;
        let exported = export_personal_catalog(&personal).unwrap();
        assert!(exported.contains("\"origin\": \"My notes\""));
        let again = parse_personal_catalog(&exported, &[]).unwrap();
        assert_eq!(again[0].name, "My Tool");
    }

    #[test]
    fn warning_policy_covers_every_curated_high_impact_entry() {
        let tools: Vec<Tool> = serde_json::from_str(include_str!("../catalog.json")).unwrap();
        let expected = [
            ("Git", "git switch <branch>"), ("Git", "git switch -c <branch>"),
            ("Git", "git add <file>"), ("Git", "git commit -m '<message>'"), ("Git", "git stash push"), ("Git", "git fetch"),
            ("SSH", "ssh <user>@<host>"), ("SSH", "ssh -p <port> <host>"), ("SSH", "ssh -i <key> <host>"), ("SSH", "ssh -v <host>"), ("SSH", "ssh -G <host>"),
            ("SSH", "ssh -L 8080:localhost:80 <host>"), ("SSH", "ssh -N -D 1080 <host>"), ("tmux", "tmux attach"),
            ("curl", "curl -v <url>"), ("curl", "curl -o <file> <url>"), ("curl", "curl --json @data.json <url>"),
            ("Vim", "dd"), ("Vim", ":w / :q / :wq"), ("fzf", "Ctrl+R"),
            ("ripgrep", "rg --hidden '<pattern>'"), ("fd", "fd -H '<pattern>'"),
            ("fd", "fd -I '<pattern>'"), ("fzf", "fzf --preview 'head -80 {}'"), ("htop", "F9 / k"),
        ];
        assert_eq!(expected.len(), 25);
        for (tool_name, key) in expected {
            let entry = tools.iter().find(|tool| tool.name == tool_name).unwrap().entries.iter().find(|entry| entry.key == key).unwrap();
            assert!(curated_warning(tool_name, &entry.key).is_some(), "missing reviewed warning for {tool_name}: {key}");
        }
        let ghostty: Vec<Shortcut> = serde_json::from_str(include_str!("../shortcuts.json")).unwrap();
        let ghostty_warnings: Vec<_> = ghostty.iter().filter(|entry| curated_warning("Ghostty", &entry.key).is_some()).collect();
        assert_eq!(ghostty_warnings.len(), 2);
        assert!(ghostty_warnings.iter().any(|entry| entry.key == "Ctrl+Shift+W"));
        assert!(ghostty_warnings.iter().any(|entry| entry.key == "Ctrl+Shift+V"));
        assert!(curated_warning("Git", "git status -sb").is_none());
        assert!(matches_sensitive(&Shortcut { key: "git switch <branch>".into(), action: "Switch branch".into(), origin: String::new(), version: String::new(), reviewed: String::new(), warning: String::new() }).unwrap().contains("placeholder"));
        assert!(curated_warning("Ghostty", "Ctrl+Shift+W").unwrap().contains("Closes"));
    }

    #[test]
    fn empty_catalog_roundtrips_and_exports_never_replace_existing_files() {
        let empty = export_personal_catalog(&[]).unwrap();
        assert_eq!(parse_personal_catalog(&empty, &[]).unwrap().len(), 0);
        let root = test_dir("exports");
        fs::create_dir_all(&root).unwrap();
        let destination = root.join("catalog.json");
        save_new(&destination, empty.as_bytes()).unwrap();
        assert_eq!(read_personal_file(&destination).unwrap(), empty);
        assert!(save_new(&destination, b"replacement").is_err());
        assert_eq!(fs::read(&destination).unwrap(), empty.as_bytes());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn atomic_persistence_rejects_symlinks_and_preserves_existing_data_on_failure() {
        let root = test_dir("symlinks");
        fs::create_dir_all(&root).unwrap();
        let real = root.join("real.json");
        let link = root.join("linked.json");
        fs::write(&real, b"original").unwrap();
        #[cfg(unix)] std::os::unix::fs::symlink(&real, &link).unwrap();
        #[cfg(unix)] {
            assert!(save_atomic(&link, b"changed").unwrap_err().contains("symlink"));
            assert_eq!(fs::read(&real).unwrap(), b"original");
        }
        save_atomic(&real, b"atomic update").unwrap();
        assert_eq!(fs::read(&real).unwrap(), b"atomic update");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn import_reader_enforces_size_and_rejects_symlink_sources() {
        let root = test_dir("imports");
        fs::create_dir_all(&root).unwrap();
        let source = root.join("oversize.json");
        fs::write(&source, vec![b' '; MAX_CATALOG_BYTES + 1]).unwrap();
        assert!(read_personal_file(&source).unwrap_err().contains("1 MiB"));
        #[cfg(unix)] {
            let link = root.join("linked.json");
            std::os::unix::fs::symlink(&source, &link).unwrap();
            assert!(read_personal_file(&link).unwrap_err().contains("symlink"));
        }
        fs::remove_dir_all(root).unwrap();
    }

    fn test_dir(name: &str) -> PathBuf {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        std::env::temp_dir().join(format!("clavis-{name}-{}-{}", std::process::id(), NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)))
    }
}
