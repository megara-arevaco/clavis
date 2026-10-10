use std::{fs, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    Spanish,
}

impl Language {
    pub fn from_index(index: u32) -> Self {
        if index == 1 { Self::Spanish } else { Self::English }
    }

    pub fn index(self) -> u32 {
        match self { Self::English => 0, Self::Spanish => 1 }
    }

    pub fn text(self, key: &str) -> &str {
        match (self, key) {
            (Self::English, "windowTitle") => "Clavis · Terminal reference",
            (Self::Spanish, "windowTitle") => "Clavis · Referencia de terminal",
            (Self::English, "toolSelector") => "Choose a terminal tool",
            (Self::Spanish, "toolSelector") => "Elige una herramienta de terminal",
            (Self::English, "hide") => "Hide reference · Esc",
            (Self::Spanish, "hide") => "Ocultar referencia · Esc",
            (Self::English, "search") => "Find a command or shortcut…",
            (Self::Spanish, "search") => "Buscar un comando o atajo…",
            (Self::English, "status") => "Double Ctrl · Esc · click to copy",
            (Self::Spanish, "status") => "Doble Ctrl · Esc · clic para copiar",
            (Self::English, "docs") => "Docs",
            (Self::Spanish, "docs") => "Ayuda",
            (Self::English, "language") => "Language",
            (Self::Spanish, "language") => "Idioma",
            (Self::English, "copyHint") => "Click to copy; never executes",
            (Self::Spanish, "copyHint") => "Clic para copiar; nunca se ejecuta",
            (Self::English, "copied") => "Copied · nothing executed",
            (Self::Spanish, "copied") => "Copiado · no se ha ejecutado nada",
            (Self::English, "empty") => "No matches. Try another word.",
            (Self::Spanish, "empty") => "No hay coincidencias. Prueba otra palabra.",
            (Self::English, "keys") => "Keys",
            (Self::Spanish, "keys") => "Teclas",
            (Self::English, "keyboardUnavailable") => "Keyboard unavailable · open from tray",
            (Self::Spanish, "keyboardUnavailable") => "Teclado no disponible · abrir desde la bandeja",
            _ => key,
        }
    }
}

fn preference_path() -> PathBuf {
    crate::input::config_path().with_file_name("language")
}

pub fn selected_language() -> Language {
    match fs::read_to_string(preference_path()).unwrap_or_default().trim() {
        "es" => Language::Spanish,
        _ => Language::English,
    }
}

pub fn remember_language(language: Language) {
    let path = preference_path();
    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_ok() {
            let value = match language { Language::English => "en", Language::Spanish => "es" };
            if let Err(error) = fs::write(path, value) {
                eprintln!("Cannot save language preference: {error}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_selector_has_stable_indices_and_translations() {
        assert_eq!(Language::from_index(0), Language::English);
        assert_eq!(Language::from_index(1), Language::Spanish);
        assert_eq!(Language::Spanish.text("empty"), "No hay coincidencias. Prueba otra palabra.");
        assert_eq!(Language::English.text("copied"), "Copied · nothing executed");
    }
}
