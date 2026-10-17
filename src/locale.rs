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
            (Self::English, "docs") => "Docs",
            (Self::Spanish, "docs") => "Ayuda",
            (Self::English, "language") => "Language",
            (Self::Spanish, "language") => "Idioma",
            (Self::English, "copied") => "Copied · nothing executed",
            (Self::Spanish, "copied") => "Copiado · no se ha ejecutado nada",
            (Self::English, "empty") => "No matches. Try another word.",
            (Self::Spanish, "empty") => "No hay coincidencias. Prueba otra palabra.",
            (Self::English, "keys") => "Keys",
            (Self::Spanish, "keys") => "Teclas",
            (Self::English, "keyboardUnavailable") => "Keyboard unavailable · open from tray",
            (Self::Spanish, "keyboardUnavailable") => "Teclado no disponible · abrir desde la bandeja",
            (Self::English, "scopeCurrent") => "This tool",
            (Self::Spanish, "scopeCurrent") => "Esta herramienta",
            (Self::English, "scopeAll") => "All tools",
            (Self::Spanish, "scopeAll") => "Todas",
            (Self::English, "searchScope") => "Choose this tool or all tools",
            (Self::Spanish, "searchScope") => "Buscar en esta herramienta o en todas",
            (Self::English, "favorites") => "Favorites",
            (Self::Spanish, "favorites") => "Favoritos",
            (Self::English, "info") => "Info",
            (Self::Spanish, "info") => "Información",
            (Self::English, "textScale") => "Text size",
            (Self::Spanish, "textScale") => "Tamaño de texto",
            (Self::English, "normal") => "Normal",
            (Self::Spanish, "normal") => "Normal",
            (Self::English, "large") => "Large",
            (Self::Spanish, "large") => "Grande",
            (Self::English, "close") => "Close",
            (Self::Spanish, "close") => "Cerrar",
            (Self::English, "importPersonal") => "Import personal JSON…",
            (Self::Spanish, "importPersonal") => "Importar JSON personal…",
            (Self::English, "exportPersonal") => "Export personal JSON…",
            (Self::Spanish, "exportPersonal") => "Exportar JSON personal…",
            (Self::English, "inputHelp") => "Input and privacy",
            (Self::Spanish, "inputHelp") => "Entrada y privacidad",
            (Self::English, "statusLabel") => "Keyboard shortcut status",
            (Self::Spanish, "statusLabel") => "Estado del atajo de teclado",
            (Self::English, "windowsStatusLabel") => "Windows keyboard listener status",
            (Self::Spanish, "windowsStatusLabel") => "Estado del listener de teclado de Windows",
            (Self::English, "safeShortcut") => "Recommended on Wayland: bind `clavis --show` in your desktop/compositor keyboard shortcuts. The compositor shortcut needs no keyboard ACL. Optional double-Ctrl monitoring is independent and continues only if separately configured; device access can expose all keystrokes to other processes running as your user.",
            (Self::Spanish, "safeShortcut") => "Recomendado en Wayland: asigna `clavis --show` en los atajos de teclado del escritorio/compositor. El atajo del compositor no requiere ACL del teclado. La detección opcional de doble Ctrl es independiente y solo sigue activa si se configuró por separado; el acceso al dispositivo puede exponer todas las pulsaciones a otros procesos de tu usuario.",
            (Self::English, "windowsSafeShortcut") => "Clavis runs in the system tray and listens for two released Ctrl taps while it is open. The listener stops when you quit Clavis. You can enable the optional login-startup shortcut in the installer.",
            (Self::Spanish, "windowsSafeShortcut") => "Clavis permanece en la bandeja y detecta dos pulsaciones y liberaciones de Ctrl mientras está abierto. El listener se detiene al cerrar Clavis. Puedes activar el inicio de sesión opcional en el instalador.",
            (Self::English, "compositorGuide") => "Read-only Wayland guide: open Settings → Keyboard → View and Customize Shortcuts → Custom Shortcuts, add `clavis --show`, then choose a key combination. Clavis does not change settings or grant keyboard access.",
            (Self::Spanish, "compositorGuide") => "Guía de Wayland, solo lectura: abre Configuración → Teclado → Ver y personalizar atajos → Atajos personalizados, añade `clavis --show` y elige una combinación. Clavis no cambia ajustes ni concede acceso al teclado.",
            (Self::English, "privacyHelp") => "Clavis never executes commands, records ordinary keystrokes, or sends telemetry. The diagnostic summary reads only Clavis configuration; permission status is not probed from this screen.",
            (Self::Spanish, "privacyHelp") => "Clavis nunca ejecuta comandos, registra pulsaciones normales ni envía telemetría. El diagnóstico solo lee la configuración de Clavis; esta pantalla no comprueba permisos.",
            (Self::English, "windowsPrivacyHelp") => "Clavis uses a Windows global keyboard hook to detect double Ctrl while it is open. It keeps only modifier state, does not store or transmit other key input, and does not execute commands.",
            (Self::Spanish, "windowsPrivacyHelp") => "Clavis usa un listener global de teclado de Windows para detectar el doble Ctrl mientras está abierto. Solo mantiene el estado de la tecla modificadora, no guarda ni transmite otras pulsaciones y no ejecuta comandos.",
            (Self::English, "notRecorded") => "Not recorded",
            (Self::Spanish, "notRecorded") => "No registrado",
            (Self::English, "globalCount") => "All-tool results",
            (Self::Spanish, "globalCount") => "Resultados de todas las herramientas",
            (Self::English, "thisToolCount") => "This-tool results",
            (Self::Spanish, "thisToolCount") => "Resultados de esta herramienta",
            (Self::English, "origin") => "Origin",
            (Self::Spanish, "origin") => "Origen",
            (Self::English, "upstreamVersion") => "Upstream version",
            (Self::Spanish, "upstreamVersion") => "Versión upstream",
            (Self::English, "reviewed") => "reviewed",
            (Self::Spanish, "reviewed") => "revisado",
            (Self::English, "source") => "Source",
            (Self::Spanish, "source") => "Fuente",
            (Self::English, "addFavorite") => "Add to favorites",
            (Self::Spanish, "addFavorite") => "Añadir a favoritos",
            (Self::English, "removeFavorite") => "Remove from favorites",
            (Self::Spanish, "removeFavorite") => "Quitar de favoritos",
            (Self::English, "warningLabel") => "Warning",
            (Self::Spanish, "warningLabel") => "Aviso",
            (Self::English, "personalTag") => "Personal",
            (Self::Spanish, "personalTag") => "Personal",
            (Self::English, "emptyFavorites") => "No favorites yet. Use the star beside a reference to save it.",
            (Self::Spanish, "emptyFavorites") => "Aún no hay favoritos. Usa la estrella junto a una referencia para guardarla.",
            (Self::English, "keyboardShortcut") => "Keyboard shortcut",
            (Self::Spanish, "keyboardShortcut") => "Atajo de teclado",
            (Self::English, "terminalCommand") => "Terminal command",
            (Self::Spanish, "terminalCommand") => "Comando de terminal",
            (Self::English, "copyHint") => "Click or press Enter to copy; never executes",
            (Self::Spanish, "copyHint") => "Clic o Enter para copiar; nunca se ejecuta",
            (Self::English, "status") => "Ctrl+F search · Esc hide · Enter copies",
            (Self::Spanish, "status") => "Ctrl+F buscar · Esc ocultar · Enter copia",
            (Self::English, "archiveBasis") => "Archived research snapshots; Clavis has not compared these references with currently installed tool versions.",
            (Self::Spanish, "archiveBasis") => "Fuentes de investigación archivadas; Clavis no las ha comparado con las versiones instaladas actualmente.",
            (Self::English, "ghosttyDefaultsBasis") => "Bundled Ghostty defaults; installed Ghostty version not verified.",
            (Self::Spanish, "ghosttyDefaultsBasis") => "Valores predeterminados incluidos de Ghostty; versión instalada no verificada.",
            (Self::English, "ghosttyConfigBasis") => "Explicit user config lines only; includes, resets and conditions are not resolved.",
            (Self::Spanish, "ghosttyConfigBasis") => "Solo líneas explícitas de configuración; no se resuelven inclusiones, reinicios ni condiciones.",
            (Self::English, "addPersonal") => "Create or edit personal…",
            (Self::Spanish, "addPersonal") => "Crear o editar personal…",
            (Self::English, "newPersonal") => "New personal reference",
            (Self::Spanish, "newPersonal") => "Nueva referencia personal",
            (Self::English, "managePersonal") => "Manage personal references",
            (Self::Spanish, "managePersonal") => "Gestionar referencias personales",
            (Self::English, "archiveStatus") => "Archived-source reference; installed version not checked",
            (Self::Spanish, "archiveStatus") => "Referencia de fuente archivada; versión instalada no comprobada",
            (Self::English, "bundledStatus") => "Bundled defaults; installed version not checked",
            (Self::Spanish, "bundledStatus") => "Valores incluidos; versión instalada no comprobada",
            (Self::English, "diagnosticTitle") => "Optional double-Ctrl device status (read-only)",
            (Self::Spanish, "diagnosticTitle") => "Estado de dispositivos para doble Ctrl (solo lectura)",
            (Self::English, "doubleCtrlNotConfigured") => "No Clavis keyboard selection is configured; double-Ctrl monitoring is inactive.",
            (Self::Spanish, "doubleCtrlNotConfigured") => "No hay una selección de teclado de Clavis; la detección de doble Ctrl está inactiva.",
            (Self::English, "isolatedDiagnostic") => "Isolated GTK test/capture mode: no keyboard devices, event state or permissions are inspected.",
            (Self::Spanish, "isolatedDiagnostic") => "Modo GTK aislado de prueba/captura: no se inspeccionan teclados, eventos ni permisos.",
            (Self::English, "refreshStatus") => "Refresh device status",
            (Self::Spanish, "refreshStatus") => "Actualizar estado de dispositivos",
            (Self::English, "listenerUpdate") => "Listener update",
            (Self::Spanish, "listenerUpdate") => "Actualización del listener",
            (Self::English, "sysfsVisible") => "visible in sysfs; permission not checked",
            (Self::Spanish, "sysfsVisible") => "visible en sysfs; permiso no comprobado",
            (Self::English, "staleDevice") => "configured path is stale; candidate",
            (Self::Spanish, "staleDevice") => "la ruta configurada está obsoleta; candidato",
            (Self::English, "candidateVisible") => "is visible (not opened)",
            (Self::Spanish, "candidateVisible") => "está visible (no se abrió)",
            (Self::English, "candidateLabel") => "candidate",
            (Self::Spanish, "candidateLabel") => "candidato",
            (Self::English, "notVisible") => "not currently visible in sysfs; reconnect state is not confirmed",
            (Self::Spanish, "notVisible") => "no está visible ahora en sysfs; reconexión sin confirmar",
            (Self::English, "devicePermissions") => "Permission is not checked. Sysfs presence does not test readability. The listener retries disconnected devices.",
            (Self::Spanish, "devicePermissions") => "No se comprueba el permiso. La presencia en sysfs no verifica lectura. El listener reintenta dispositivos desconectados.",
            (Self::English, "checkInputHelp") => "`clavis --check-input` opens configured devices but does not capture key events.",
            (Self::Spanish, "checkInputHelp") => "`clavis --check-input` abre dispositivos configurados, pero no captura eventos de teclas.",
            (Self::English, "windowsKeyboardListener") => "Windows global keyboard hook is active. Clavis checks Ctrl press and release state to detect double Ctrl; it does not record or transmit ordinary keystrokes. Use the system tray to show, hide or quit Clavis.",
            (Self::Spanish, "windowsKeyboardListener") => "El listener global de teclado de Windows está activo. Clavis comprueba las pulsaciones y liberaciones de Ctrl para detectar el doble Ctrl; no registra ni transmite otras teclas. Usa la bandeja del sistema para mostrar, ocultar o cerrar Clavis.",
            (Self::English, "editPersonal") => "Edit personal reference…",
            (Self::Spanish, "editPersonal") => "Editar referencia personal…",
            (Self::English, "personalEditorTitle") => "Personal reference",
            (Self::Spanish, "personalEditorTitle") => "Referencia personal",
            (Self::English, "editorFieldsHelp") => "Required: name, HTTPS documentation source, origin, version or “Not recorded”, review date (YYYY-MM-DD or “Not recorded”), and one or more entries. Each entry is one line: key | explanation | optional warning. Escape a literal | as \\|.",
            (Self::Spanish, "editorFieldsHelp") => "Obligatorios: nombre, fuente HTTPS, origen, versión o «No registrado», fecha de revisión (AAAA-MM-DD o «No registrado») y una o más entradas. Cada línea: tecla/comando | explicación | aviso opcional. Escapa un | literal como \\|.",
            (Self::English, "editorCreateChoice") => "Create new reference",
            (Self::Spanish, "editorCreateChoice") => "Crear referencia nueva",
            (Self::English, "save") => "Save",
            (Self::Spanish, "save") => "Guardar",
            (Self::English, "cancel") => "Cancel",
            (Self::Spanish, "cancel") => "Cancelar",
            (Self::English, "name") => "Name",
            (Self::Spanish, "name") => "Nombre",
            (Self::English, "kind") => "Type",
            (Self::Spanish, "kind") => "Tipo",
            (Self::English, "documentationUrl") => "Documentation URL (HTTPS)",
            (Self::Spanish, "documentationUrl") => "URL de documentación (HTTPS)",
            (Self::English, "note") => "Usage note",
            (Self::Spanish, "note") => "Nota de uso",
            (Self::English, "originField") => "Origin",
            (Self::Spanish, "originField") => "Origen",
            (Self::English, "versionValue") => "Upstream version",
            (Self::Spanish, "versionValue") => "Versión upstream",
            (Self::English, "reviewDate") => "Review date",
            (Self::Spanish, "reviewDate") => "Fecha de revisión",
            (Self::English, "entries") => "Entries",
            (Self::Spanish, "entries") => "Entradas",
            (Self::English, "personalSaved") => "Personal reference saved.",
            (Self::Spanish, "personalSaved") => "Referencia personal guardada.",
            (Self::English, "personalBasis") => "Personal source; version and review metadata supplied by its author.",
            (Self::Spanish, "personalBasis") => "Fuente personal; versión y revisión indicadas por quien la creó.",
            (Self::English, "readOnlyDiag") => "Read-only guidance; Clavis does not configure compositor shortcuts or grant access.",
            (Self::Spanish, "readOnlyDiag") => "Orientación de solo lectura; Clavis no configura atajos del compositor ni concede acceso.",
            (Self::English, "exportEmpty") => "Empty personal catalog exported as valid schema version 1 JSON.",
            (Self::Spanish, "exportEmpty") => "Catálogo personal vacío exportado como JSON válido de esquema 1.",
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

pub fn remember_language(language: Language) -> Result<(), String> {
    let path = preference_path();
    let value = match language { Language::English => "en", Language::Spanish => "es" };
    crate::catalog::save_atomic(&path, value.as_bytes())
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
