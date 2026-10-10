use crate::catalog::{self, Tool};
use crate::locale::{self, Language};
use gtk::{glib, prelude::*};
use std::{cell::{Cell, RefCell}, fs, rc::Rc};

static GTK_TEST_SUCCEEDED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub struct Ui {
    pub window: gtk::ApplicationWindow,
    pub selector: gtk::DropDown,
    pub search: gtk::SearchEntry,
    pub status: gtk::Label,
    pub diagnostic: gtk::Label,
    pub language: Rc<Cell<Language>>,
}

pub fn build(app: &gtk::Application, tools: Rc<RefCell<Vec<Tool>>>, test_mode: bool, isolated_mode: bool, capture_width: Option<i32>) -> Ui {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("style.css"));
    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
    let initial_language = locale::selected_language();
    let language = Rc::new(Cell::new(initial_language));
    let window = gtk::ApplicationWindow::builder().application(app)
        .title(initial_language.text("windowTitle")).default_width(capture_width.unwrap_or(560)).default_height(440)
        .decorated(false).build();
    window.set_icon_name(Some("clavis"));
    window.add_css_class("overlay");
    window.connect_map(|_| eprintln!("Overlay mapped by GTK"));
    window.connect_unmap(|_| eprintln!("Overlay hidden"));

    let body = gtk::Box::new(gtk::Orientation::Vertical, 7);
    body.set_margin_top(10); body.set_margin_bottom(9);
    body.set_margin_start(12); body.set_margin_end(12);
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 7);
    let names = tool_names(&tools);
    let selector = gtk::DropDown::from_strings(&names.iter().map(String::as_str).collect::<Vec<_>>());
    selector.set_tooltip_text(Some(initial_language.text("toolSelector")));
    selector.update_property(&[gtk::accessible::Property::Label(initial_language.text("toolSelector"))]);
    selector.set_hexpand(true);
    selector.set_selected(catalog::selected_tool(&tools.borrow()));
    header.append(&selector);
    let kind = gtk::Label::new(None); kind.add_css_class("muted"); header.append(&kind);
    let language_selector = gtk::DropDown::from_strings(&["English", "Español"]);
    language_selector.set_selected(initial_language.index());
    language_selector.set_size_request(98, -1);
    language_selector.set_tooltip_text(Some(initial_language.text("language")));
    language_selector.update_property(&[gtk::accessible::Property::Label(initial_language.text("language"))]);
    header.append(&language_selector);
    let info_button = gtk::Button::with_label(initial_language.text("info"));
    info_button.add_css_class("flat");
    info_button.set_tooltip_text(Some(initial_language.text("inputHelp")));
    header.append(&info_button);
    let hide = gtk::Button::from_icon_name("window-close-symbolic");
    hide.set_tooltip_text(Some(initial_language.text("hide")));
    hide.update_property(&[gtk::accessible::Property::Label(initial_language.text("hide"))]);
    hide.add_css_class("flat");
    let weak = window.downgrade();
    hide.connect_clicked(move |_| { if let Some(window) = weak.upgrade() { window.set_visible(false); } });
    header.append(&hide); body.append(&header);

    let search_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some(initial_language.text("search")));
    search.set_hexpand(true);
    search.set_tooltip_text(Some(initial_language.text("search")));
    search.update_property(&[gtk::accessible::Property::Label(initial_language.text("search"))]);
    search_row.append(&search);
    let scope = gtk::DropDown::from_strings(&[
        initial_language.text("scopeCurrent"), initial_language.text("scopeAll")
    ]);
    scope.set_selected(0);
    scope.set_size_request(112, -1);
    scope.set_tooltip_text(Some(initial_language.text("searchScope")));
    scope.update_property(&[gtk::accessible::Property::Label(initial_language.text("searchScope"))]);
    search_row.append(&scope);
    let favorites_only = gtk::ToggleButton::with_label(initial_language.text("favorites"));
    favorites_only.set_tooltip_text(Some("Show saved references only"));
    favorites_only.add_css_class("flat");
    search_row.append(&favorites_only);
    body.append(&search_row);

    let list = gtk::Grid::builder().column_homogeneous(true).column_spacing(12).row_spacing(2).build();
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

    let diagnostic = gtk::Label::new(None);
    diagnostic.set_xalign(0.0); diagnostic.set_yalign(0.0); diagnostic.set_wrap(true);
    diagnostic.set_selectable(true); diagnostic.add_css_class("help-copy");
    let privacy = gtk::Label::new(Some(initial_language.text("privacyHelp")));
    privacy.set_xalign(0.0); privacy.set_wrap(true); privacy.add_css_class("muted");
    let safe_shortcut = gtk::Label::new(Some(initial_language.text("safeShortcut")));
    safe_shortcut.set_xalign(0.0); safe_shortcut.set_wrap(true); safe_shortcut.add_css_class("help-copy");
    let source_info = gtk::Label::new(None);
    source_info.set_xalign(0.0); source_info.set_yalign(0.0); source_info.set_wrap(true);
    source_info.set_selectable(true); source_info.add_css_class("help-copy");
    let scale_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let scale_label = gtk::Label::new(Some(initial_language.text("textScale")));
    scale_label.set_xalign(0.0); scale_label.set_hexpand(true); scale_row.append(&scale_label);
    let scale = gtk::DropDown::from_strings(&[initial_language.text("normal"), initial_language.text("large")]);
    scale.set_size_request(110, -1);
    scale_label.set_mnemonic_widget(Some(&scale));
    scale.update_property(&[gtk::accessible::Property::Label(initial_language.text("textScale"))]);
    let scale_path = crate::input::config_path().with_file_name("text-size");
    let large_text = Rc::new(Cell::new(fs::read_to_string(&scale_path).is_ok_and(|value| value.trim() == "large")));
    scale.set_selected(u32::from(large_text.get()));
    if large_text.get() { window.add_css_class("large-text"); }
    scale_row.append(&scale);

    let dialog = gtk::Window::builder().title(initial_language.text("inputHelp"))
        .transient_for(&window).modal(true).default_width(460).default_height(460).resizable(true).build();
    dialog.add_css_class("overlay");
    let dialog_body = gtk::Box::new(gtk::Orientation::Vertical, 10);
    dialog_body.set_margin_top(14); dialog_body.set_margin_bottom(12);
    dialog_body.set_margin_start(14); dialog_body.set_margin_end(14);
    let reading = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let status_title = gtk::Label::new(Some(initial_language.text("statusLabel")));
    status_title.set_xalign(0.0); status_title.add_css_class("dialog-title");
    reading.append(&status_title); reading.append(&diagnostic);
    let refresh_status = gtk::Button::with_label(initial_language.text("refreshStatus"));
    refresh_status.set_halign(gtk::Align::Start);
    reading.append(&refresh_status);
    reading.append(&safe_shortcut); reading.append(&privacy);
    reading.append(&source_info); reading.append(&scale_row);
    let reading_scroll = gtk::ScrolledWindow::builder().vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never).child(&reading).build();
    dialog_body.append(&reading_scroll);
    let personal_actions = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let manage_actions = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let manage_button = gtk::Button::with_label(initial_language.text("managePersonal"));
    let editor_test_slot = Rc::new(RefCell::new(None::<PersonalEditorWidgets>));
    let import_button = gtk::Button::with_label(initial_language.text("importPersonal"));
    let export_button = gtk::Button::with_label(initial_language.text("exportPersonal"));
    manage_actions.append(&manage_button);
    personal_actions.append(&manage_actions);
    let file_actions = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    file_actions.append(&import_button); file_actions.append(&export_button);
    personal_actions.append(&file_actions);
    dialog_body.append(&personal_actions);
    let close_dialog = gtk::Button::with_label(initial_language.text("close"));
    close_dialog.set_halign(gtk::Align::End); dialog_body.append(&close_dialog);
    dialog.set_child(Some(&dialog_body));
    let weak_dialog = dialog.downgrade();
    close_dialog.connect_clicked(move |_| { if let Some(dialog) = weak_dialog.upgrade() { dialog.set_visible(false); } });
    let weak_dialog = dialog.downgrade();
    info_button.connect_clicked(move |_| { if let Some(dialog) = weak_dialog.upgrade() { dialog.present(); } });
    let diagnostic_refresh = diagnostic.clone(); let language_refresh = language.clone();
    refresh_status.connect_clicked(move |_| {
        let summary = if isolated_mode { crate::input::isolated_diagnostic(language_refresh.get()) }
            else { crate::input::diagnostic_summary(language_refresh.get()) };
        diagnostic_refresh.set_text(&summary);
        diagnostic_refresh.set_tooltip_text(Some(&summary));
    });

    let favorites = Rc::new(RefCell::new(catalog::load_favorites()));
    let first_copy = Rc::new(RefCell::new(None::<gtk::Button>));
    let render: Rc<dyn Fn()> = {
        let tools = tools.clone(); let language = language.clone();
        let status = status.clone(); let kind = kind.clone(); let docs = docs.clone();
        let list = list.clone(); let content_scroll = scroll.clone(); let scope = scope.clone(); let selector = selector.clone();
        let search = search.clone(); let favorites_only = favorites_only.clone();
        let favorites = favorites.clone(); let large_text = large_text.clone();
        let source_info = source_info.clone(); let first_copy = first_copy.clone();
        let render_window = window.clone();
        Rc::new(move || {
            while let Some(child) = list.first_child() { list.remove(&child); }
            first_copy.borrow_mut().take();
            let locale = language.get();
            let current_tools = tools.borrow();
            let Some(selected_tool) = current_tools.get(selector.selected() as usize) else { return; };
            kind.set_text(if selected_tool.kind == "Keys" { locale.text("keyboardShortcut") } else { locale.text("terminalCommand") });
            docs.set_uri(&selected_tool.source);
            docs.set_tooltip_text(Some(&selected_tool.note));
            set_source_info(&source_info, selected_tool, locale);
            let query = search.text().to_string();
            let all_tools = scope.selected() == 1;
            let favorites_filter = favorites_only.is_active();
            let favorite_set = favorites.borrow();
            let width = if content_scroll.width() > 0 { content_scroll.width() } else { render_window.default_width().max(360) - 24 };
            let columns = columns_for_width(width, large_text.get());
            list.set_column_spacing(if columns == 1 { 0 } else { 12 });
            let mut count = 0usize;
            for (tool_index, tool) in current_tools.iter().enumerate() {
                if !all_tools && tool_index != selector.selected() as usize { continue; }
                let tool_matches_name = !query.trim().is_empty() && tool.name.to_lowercase().contains(&query.to_lowercase());
                for entry in &tool.entries {
                    let favorite_key = catalog::favorite_id(&tool.name, &entry.key);
                    let is_favorite = favorite_set.contains(&favorite_key);
                    if favorites_filter && !is_favorite { continue; }
                    if !tool_matches_name && !catalog::matches(entry, &query) { continue; }
                    count += 1;
                    let line = gtk::Box::new(gtk::Orientation::Horizontal, 4);
                    line.set_hexpand(true);
                    let button = gtk::Button::new(); button.add_css_class("reference-row");
                    button.set_hexpand(true);
                    let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
                    if all_tools {
                        let personal_tag = if tool.personal { format!(" · {}", locale.text("personalTag")) } else { String::new() };
                        let origin = gtk::Label::new(Some(&format!("{}{} · {}", tool.name, personal_tag, if tool.kind == "Keys" { locale.text("keyboardShortcut") } else { locale.text("terminalCommand") })));
                        origin.set_xalign(0.0); origin.set_ellipsize(gtk::pango::EllipsizeMode::End); origin.add_css_class("source-label");
                        content.append(&origin);
                    }
                    let key = gtk::Label::new(None);
                    key.set_markup(&highlight_placeholders(&entry.key));
                    key.set_xalign(0.0); key.set_wrap(true); key.set_wrap_mode(gtk::pango::WrapMode::WordChar);
                    key.set_max_width_chars(48); key.add_css_class("key");
                    let action = gtk::Label::new(Some(&entry.action));
                    action.set_xalign(0.0); action.set_wrap(true); action.set_hexpand(true); action.add_css_class("muted");
                    content.append(&key); content.append(&action);
                    if all_tools || tool.personal {
                        let basis = reference_basis(tool, locale);
                        let provenance = gtk::Label::new(Some(&format!("{} · v{} · {} {} · {basis}", entry.origin, entry.version, locale.text("reviewed"), entry.reviewed)));
                        provenance.set_xalign(0.0); provenance.set_wrap(true); provenance.add_css_class("source-label");
                        content.append(&provenance);
                    }
                    let warning = catalog::curated_warning(&tool.name, &entry.key)
                        .map(str::to_string).or_else(|| catalog::matches_sensitive(entry));
                    if let Some(warning) = warning {
                        let caution = gtk::Label::new(Some(&format!("{}: {warning}", locale.text("warningLabel"))));
                        caution.set_xalign(0.0); caution.set_wrap(true); caution.add_css_class("warning");
                        content.append(&caution);
                    }
                    button.set_tooltip_text(Some(&format!("{}\n{}\n{}\n{}", entry.key, entry.action, entry.origin, locale.text("copyHint"))));
                    button.update_property(&[gtk::accessible::Property::Label(&format!("{} — {}", entry.key, entry.action))]);
                    button.set_child(Some(&content));
                    let value = entry.key.clone(); let copy_status = status.clone();
                    let copied_message = locale.text("copied").to_string();
                    button.connect_clicked(move |_| {
                        gtk::gdk::Display::default().map(|display| display.clipboard().set_text(&value));
                        copy_status.set_text(&copied_message);
                    });
                    if first_copy.borrow().is_none() { *first_copy.borrow_mut() = Some(button.clone()); }
                    line.append(&button);
                    let star = gtk::ToggleButton::new();
                    star.set_child(Some(&gtk::Image::from_icon_name(if is_favorite { "starred-symbolic" } else { "non-starred-symbolic" })));
                    star.add_css_class("favorite-toggle"); star.set_active(is_favorite);
                    star.set_valign(gtk::Align::Start);
                    let star_label = locale.text(if is_favorite { "removeFavorite" } else { "addFavorite" });
                    star.set_tooltip_text(Some(star_label));
                    star.update_property(&[gtk::accessible::Property::Label(star_label)]);
                    let favorites = favorites.clone(); let status = status.clone();
                    let tool_name = tool.name.clone(); let key_value = entry.key.clone();
                    let is_filter = favorites_filter; let locale = locale;
                    let saving = Rc::new(Cell::new(false));
                    let saving_toggle = saving.clone();
                    star.connect_toggled(move |star| {
                        if saving_toggle.get() { return; }
                        let active = star.is_active();
                        let id = catalog::favorite_id(&tool_name, &key_value);
                        let mut saved = favorites.borrow_mut();
                        let previous = saved.clone();
                        if active { saved.insert(id); } else { saved.remove(&id); }
                        let result = catalog::save_favorites(&saved);
                        if let Err(error) = result {
                            *saved = previous;
                            drop(saved);
                            saving_toggle.set(true); star.set_active(!active); saving_toggle.set(false);
                            status.set_text(&format!("Could not save favorites: {error}"));
                            return;
                        }
                        drop(saved);
                        star.set_child(Some(&gtk::Image::from_icon_name(if active { "starred-symbolic" } else { "non-starred-symbolic" })));
                        let star_label = locale.text(if active { "removeFavorite" } else { "addFavorite" });
                        star.set_tooltip_text(Some(star_label));
                        star.update_property(&[gtk::accessible::Property::Label(star_label)]);
                        if is_filter && !active {
                            if let Some(row) = star.parent() { row.set_visible(false); }
                        }
                    });
                    line.append(&star);
                    let (column, row) = cell_position(count - 1, columns);
                    list.attach(&line, column, row, 1, 1);
                }
            }
            if count == 0 {
                let message = if favorites_filter && !query.is_empty() {
                    locale.text("empty")
                } else if favorites_filter { locale.text("emptyFavorites") } else { locale.text("empty") };
                let empty = gtk::Label::new(Some(message));
                empty.add_css_class("muted"); empty.set_margin_top(22); empty.set_wrap(true);
                list.attach(&empty, 0, 0, columns as i32, 1);
            }
            if favorites_filter {
                status.set_text(&format!("{}: {count}", locale.text("favorites")));
            } else if all_tools && !query.trim().is_empty() {
                status.set_text(&format!("{}: {count}", locale.text("globalCount")));
            } else if !query.trim().is_empty() {
                status.set_text(&format!("{}: {count}", locale.text("thisToolCount")));
            } else {
                status.set_text(locale.text("status"));
            }
        })
    };
    render();

    let update = render.clone(); let tools_for_selection = tools.clone(); let tool_status = status.clone();
    selector.connect_selected_notify(move |selector| {
        let save_error = tools_for_selection.borrow().get(selector.selected() as usize)
            .and_then(|tool| catalog::remember_tool(&tool.name).err());
        update();
        if let Some(error) = save_error { tool_status.set_text(&format!("Could not save selected tool: {error}")); }
    });
    let update = render.clone();
    search.connect_search_changed(move |_| update());
    let first_copy_on_enter = first_copy.clone();
    search.connect_activate(move |search| {
        if !search.text().trim().is_empty() {
            if let Some(button) = first_copy_on_enter.borrow().as_ref() { button.emit_clicked(); }
        }
    });
    let update = render.clone();
    scope.connect_selected_notify(move |_| update());
    let update = render.clone();
    favorites_only.connect_toggled(move |_| update());
    let update = render.clone();
    window.connect_notify_local(Some("width"), move |_, _| update());
    let update = render.clone();
    scroll.connect_notify_local(Some("width"), move |_, _| update());

    let large_text_change = large_text.clone();
    let update = render.clone();
    let window_scale = window.clone(); let scale_status = status.clone(); let scale_path_for_save = scale_path.clone();
    scale.connect_selected_notify(move |scale| {
        let large = scale.selected() == 1;
        large_text_change.set(large);
        if large { window_scale.add_css_class("large-text"); } else { window_scale.remove_css_class("large-text"); }
        let preference = if large { "large" } else { "normal" };
        let save_error = catalog::save_atomic(&scale_path_for_save, preference.as_bytes()).err();
        update();
        if let Some(error) = save_error { scale_status.set_text(&format!("Could not save text size preference: {error}")); }
    });

    let update = render.clone();
    let tools_for_editor = tools.clone();
    let selector_for_editor = selector.clone();
    let status_for_editor = status.clone();
    let editor_parent = window.clone();
    let editor_language = language.clone(); let editor_slot = editor_test_slot.clone();
    manage_button.connect_clicked(move |_| {
        let editor = open_personal_editor(
            &editor_parent, tools_for_editor.clone(), selector_for_editor.clone(),
            status_for_editor.clone(), update.clone(), editor_language.get(),
        );
        *editor_slot.borrow_mut() = Some(editor);
    });

    let update = render.clone();
    let status_for_import = status.clone();
    let tools_for_import = tools.clone();
    let selector_for_import = selector.clone();
    let parent = window.clone();
    import_button.connect_clicked(move |_| {
        let chooser = gtk::FileChooserNative::new(
            Some("Import personal references"), Some(&parent), gtk::FileChooserAction::Open,
            Some("Import"), Some("Cancel"));
        let tools = tools_for_import.clone(); let selector = selector_for_import.clone();
        let status = status_for_import.clone(); let update = update.clone();
        chooser.connect_response(move |chooser, response| {
            if response == gtk::ResponseType::Accept {
                let result = chooser.file().and_then(|file| file.path()).ok_or_else(|| "Choose a local JSON file.".to_string())
                    .and_then(|path| catalog::read_personal_file(&path))
                    .and_then(|contents| {
                        let selected_name = tools.borrow().get(selector.selected() as usize).map(|tool| tool.name.clone());
                        let import_count = catalog::import_personal_contents(&contents, &mut tools.borrow_mut())?;
                        let names = tool_names(&tools);
                        let model = gtk::StringList::new(&names.iter().map(String::as_str).collect::<Vec<_>>());
                        selector.set_model(Some(&model));
                        if let Some(name) = selected_name {
                            let index = tools.borrow().iter().position(|tool| tool.name == name).unwrap_or(0);
                            selector.set_selected(index as u32);
                        }
                        update();
                        Ok(format!("Imported {import_count} personal reference(s); existing entries were kept."))
                    });
                match result { Ok(message) => status.set_text(&message), Err(error) => status.set_text(&format!("Import rejected: {error}")) }
            }
            chooser.destroy();
        });
        chooser.show();
    });

    let parent = window.clone(); let tools_for_export = tools.clone(); let status_for_export = status.clone();
    export_button.connect_clicked(move |_| {
        let chooser = gtk::FileChooserNative::new(
            Some("Export personal references"), Some(&parent), gtk::FileChooserAction::Save,
            Some("Export"), Some("Cancel"));
        chooser.set_current_name("clavis-personal.json");
        let tools = tools_for_export.clone(); let status = status_for_export.clone();
        chooser.connect_response(move |chooser, response| {
            if response == gtk::ResponseType::Accept {
                let output = catalog::export_personal_catalog(&tools.borrow());
                let result = output.and_then(|contents| {
                    let empty = contents.contains("\"references\": []");
                    let path = chooser.file().and_then(|file| file.path()).ok_or_else(|| "Choose a local destination".to_string())?;
                    catalog::save_new(&path, contents.as_bytes()).map(|()| empty)
                });
                match result {
                    Ok(true) => status.set_text("Empty personal catalog exported as valid schema version 1 JSON."),
                    Ok(false) => status.set_text("Personal references exported as validated JSON."),
                    Err(error) => status.set_text(&format!("Export unavailable: {error}")),
                }
            }
            chooser.destroy();
        });
        chooser.show();
    });

    let weak_search = search.downgrade(); let weak_docs = docs.downgrade();
    let weak_manage = manage_button.downgrade();
    let weak_window = window.downgrade(); let weak_info = info_button.downgrade();
    let weak_hide = hide.downgrade(); let weak_scope = scope.downgrade();
    let weak_language_selector = language_selector.downgrade();
    let weak_fav = favorites_only.downgrade(); let weak_status_title = status_title.downgrade();
    let weak_refresh = refresh_status.downgrade();
    let weak_status_line = status.downgrade();
    let weak_privacy = privacy.downgrade(); let weak_safe = safe_shortcut.downgrade();
    let weak_scale_label = scale_label.downgrade(); let weak_scale = scale.downgrade(); let weak_import = import_button.downgrade();
    let weak_export = export_button.downgrade(); let weak_close = close_dialog.downgrade();
    let weak_dialog = dialog.downgrade(); let language_state = language.clone(); let update = render.clone();
    language_selector.connect_selected_notify(move |selector| {
        let selected_language = Language::from_index(selector.selected());
        language_state.set(selected_language);
        let language_save_error = locale::remember_language(selected_language).err();
        if let Some(window) = weak_window.upgrade() { window.set_title(Some(selected_language.text("windowTitle"))); }
        if let Some(search) = weak_search.upgrade() { search.set_placeholder_text(Some(selected_language.text("search"))); search.set_tooltip_text(Some(selected_language.text("search"))); }
        if let Some(scope) = weak_scope.upgrade() {
            let active = scope.selected();
            let model = gtk::StringList::new(&[selected_language.text("scopeCurrent"), selected_language.text("scopeAll")]);
            scope.set_model(Some(&model)); scope.set_selected(active);
            scope.set_tooltip_text(Some(selected_language.text("searchScope")));
            scope.update_property(&[gtk::accessible::Property::Label(selected_language.text("searchScope"))]);
        }
        if let Some(language_selector) = weak_language_selector.upgrade() {
            language_selector.update_property(&[gtk::accessible::Property::Label(selected_language.text("language"))]);
        }
        if let Some(hide) = weak_hide.upgrade() { hide.update_property(&[gtk::accessible::Property::Label(selected_language.text("hide"))]); }
        if let Some(search) = weak_search.upgrade() { search.update_property(&[gtk::accessible::Property::Label(selected_language.text("search"))]); }
        if let Some(scale) = weak_scale.upgrade() {
            let active = scale.selected();
            let model = gtk::StringList::new(&[selected_language.text("normal"), selected_language.text("large")]);
            scale.set_model(Some(&model)); scale.set_selected(active);
        }
        if let Some(favorites) = weak_fav.upgrade() { favorites.set_label(selected_language.text("favorites")); }
        if let Some(docs) = weak_docs.upgrade() { docs.set_label(selected_language.text("docs")); }
        if let Some(info) = weak_info.upgrade() { info.set_label(selected_language.text("info")); info.set_tooltip_text(Some(selected_language.text("inputHelp"))); }
        if let Some(hide) = weak_hide.upgrade() { hide.set_tooltip_text(Some(selected_language.text("hide"))); }
        if let Some(label) = weak_status_title.upgrade() { label.set_text(selected_language.text("statusLabel")); }
        if let Some(button) = weak_refresh.upgrade() { button.set_label(selected_language.text("refreshStatus")); }
        if let Some(label) = weak_privacy.upgrade() { label.set_text(selected_language.text("privacyHelp")); }
        if let Some(label) = weak_safe.upgrade() { label.set_text(selected_language.text("safeShortcut")); }
        if let Some(label) = weak_scale_label.upgrade() { label.set_text(selected_language.text("textScale")); }
        if let Some(button) = weak_manage.upgrade() { button.set_label(selected_language.text("managePersonal")); }
        if let Some(button) = weak_import.upgrade() { button.set_label(selected_language.text("importPersonal")); }
        if let Some(button) = weak_export.upgrade() { button.set_label(selected_language.text("exportPersonal")); }
        if let Some(button) = weak_close.upgrade() { button.set_label(selected_language.text("close")); }
        if let Some(dialog) = weak_dialog.upgrade() { dialog.set_title(Some(selected_language.text("inputHelp"))); }
        update();
        if let Some(error) = language_save_error { if let Some(status) = weak_status_line.upgrade() { status.set_text(&format!("Could not save language preference: {error}")); } }
    });
    window.connect_close_request(|window| { window.set_visible(false); glib::Propagation::Stop });
    let keys = gtk::EventControllerKey::new();
    let weak = window.downgrade(); let search_focus = search.clone();
    keys.connect_key_pressed(move |_, key, _, state| {
        weak.upgrade().map_or(glib::Propagation::Proceed, |window| handle_window_key(&window, &search_focus, key, state))
    });
    window.add_controller(keys);
    let search_on_map = search.clone(); let render_on_map = render.clone();
    let scroll_for_resize = scroll.clone(); let window_for_resize = window.downgrade();
    let last_scroll_width = Rc::new(Cell::new(-1));
    let last_width = last_scroll_width.clone();
    window.connect_map(move |_| {
        search_on_map.grab_focus(); render_on_map();
        let weak_window = window_for_resize.clone(); let scroll = scroll_for_resize.clone();
        let update = render_on_map.clone(); let last_width = last_width.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
            if weak_window.upgrade().is_none_or(|window| !window.is_visible()) { return glib::ControlFlow::Break; }
            let width = scroll.width();
            if width != last_width.replace(width) { update(); }
            glib::ControlFlow::Continue
        });
    });
    if test_mode {
        let app = app.clone();
        let controls = SmokeControls {
            window: window.clone(), search: search.clone(), scope: scope.clone(),
            favorites_only: favorites_only.clone(), language_selector: language_selector.clone(), scale: scale.clone(),
            manage_button: manage_button.clone(), editor_slot: editor_test_slot.clone(),
            info_button: info_button.clone(), refresh_status: refresh_status.clone(), diagnostic: diagnostic.clone(),
            close_dialog: close_dialog.clone(), dialog: dialog.clone(), list: list.clone(), status: status.clone(),
            language: language.clone(), tools: tools.clone(),
        };
        let test_path = scale_path.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(350), move || {
            let success = run_gtk_smoke_test(&controls, &test_path);
            GTK_TEST_SUCCEEDED.store(success, std::sync::atomic::Ordering::SeqCst);
            app.quit();
        });
    }
    Ui { window, selector, search, status, diagnostic, language }
}

fn tool_names(tools: &Rc<RefCell<Vec<Tool>>>) -> Vec<String> {
    tools.borrow().iter().map(|tool| tool.name.clone()).collect()
}

#[derive(Clone)]
struct PersonalEditorWidgets {
    window: gtk::Window,
    name: gtk::Entry,
    source: gtk::Entry,
    note: gtk::Entry,
    origin: gtk::Entry,
    version: gtk::Entry,
    reviewed: gtk::Entry,
    entries: gtk::TextView,
    choice: gtk::DropDown,
    save: gtk::Button,
}

fn open_personal_editor(
    parent: &gtk::ApplicationWindow,
    tools: Rc<RefCell<Vec<Tool>>>,
    selector: gtk::DropDown,
    status: gtk::Label,
    update: Rc<dyn Fn()>,
    language: Language,
) -> PersonalEditorWidgets {
    let personal_indices: Vec<usize> = tools.borrow().iter().enumerate()
        .filter_map(|(index, tool)| tool.personal.then_some(index)).collect();
    let mut choices = vec![language.text("editorCreateChoice").to_string()];
    choices.extend(personal_indices.iter().map(|index| tools.borrow()[*index].name.clone()));

    let editor = gtk::Window::builder().title(language.text("personalEditorTitle"))
        .transient_for(parent).modal(true).default_width(520).default_height(620).resizable(true).build();
    editor.add_css_class("overlay");
    let outer = gtk::Box::new(gtk::Orientation::Vertical, 8);
    outer.set_margin_top(12); outer.set_margin_bottom(10); outer.set_margin_start(14); outer.set_margin_end(14);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let help = gtk::Label::new(Some(language.text("editorFieldsHelp")));
    help.set_xalign(0.0); help.set_wrap(true); help.add_css_class("muted");
    body.append(&help);
    let grid = gtk::Grid::builder().column_spacing(10).row_spacing(8).build();
    let choice = gtk::DropDown::from_strings(&choices.iter().map(|value| value.as_str()).collect::<Vec<_>>());
    add_editor_row(&grid, 0, language.text("editPersonal"), &choice);
    let name = gtk::Entry::new(); name.set_placeholder_text(Some("My tool")); add_editor_row(&grid, 1, language.text("name"), &name);
    let kind = gtk::DropDown::from_strings(&["Commands", "Keys"]); add_editor_row(&grid, 2, language.text("kind"), &kind);
    let source = gtk::Entry::new(); source.set_placeholder_text(Some("https://…")); add_editor_row(&grid, 3, language.text("documentationUrl"), &source);
    let note = gtk::Entry::new(); add_editor_row(&grid, 4, language.text("note"), &note);
    let origin = gtk::Entry::new(); origin.set_text("Personal notes"); add_editor_row(&grid, 5, language.text("originField"), &origin);
    let version = gtk::Entry::new(); version.set_text("Not recorded"); add_editor_row(&grid, 6, language.text("versionValue"), &version);
    let reviewed = gtk::Entry::new(); reviewed.set_text("Not recorded"); add_editor_row(&grid, 7, language.text("reviewDate"), &reviewed);
    body.append(&grid);
    let entries_label = gtk::Label::new(Some(language.text("entries")));
    entries_label.set_xalign(0.0); body.append(&entries_label);
    let entries = gtk::TextView::new();
    entries_label.set_mnemonic_widget(Some(&entries));
    entries.set_wrap_mode(gtk::WrapMode::WordChar);
    entries.set_size_request(-1, 150);
    entries.set_tooltip_text(Some(language.text("editorFieldsHelp")));
    let entries_scroll = gtk::ScrolledWindow::builder().min_content_height(150).child(&entries).build();
    body.append(&entries_scroll);
    let editor_scroll = gtk::ScrolledWindow::builder().vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never).child(&body).build();
    outer.append(&editor_scroll);
    let error = gtk::Label::new(None);
    error.set_xalign(0.0); error.set_wrap(true); error.add_css_class("warning");
    outer.append(&error);
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let cancel = gtk::Button::with_label(language.text("cancel"));
    let save = gtk::Button::with_label(language.text("save")); save.set_halign(gtk::Align::End);
    actions.append(&cancel); actions.append(&save); outer.append(&actions);
    editor.set_child(Some(&outer));

    let selected_index = Rc::new(Cell::new(None::<usize>));
    let name_fill = name.clone(); let kind_fill = kind.clone(); let source_fill = source.clone();
    let note_fill = note.clone(); let origin_fill = origin.clone(); let version_fill = version.clone();
    let reviewed_fill = reviewed.clone(); let entries_fill = entries.clone(); let selected_fill = selected_index.clone();
    let tools_fill = tools.clone();
    choice.connect_selected_notify(move |choice| {
        let Some(tool_index) = choice.selected().checked_sub(1).and_then(|index| personal_indices.get(index as usize)).copied() else {
            selected_fill.set(None);
            name_fill.set_text(""); kind_fill.set_selected(0); source_fill.set_text("");
            note_fill.set_text(""); origin_fill.set_text("Personal notes"); version_fill.set_text("Not recorded");
            reviewed_fill.set_text("Not recorded"); entries_fill.buffer().set_text("");
            return;
        };
        selected_fill.set(Some(tool_index));
        if let Some(tool) = tools_fill.borrow().get(tool_index) {
            name_fill.set_text(&tool.name); kind_fill.set_selected(u32::from(tool.kind == "Keys"));
            source_fill.set_text(&tool.source); note_fill.set_text(&tool.note); origin_fill.set_text(&tool.origin);
            version_fill.set_text(&tool.version); reviewed_fill.set_text(&tool.reviewed);
            let contents = tool.entries.iter().map(|entry| {
                let key = escape_editor_field(&entry.key);
                let action = escape_editor_field(&entry.action);
                if entry.warning.is_empty() { format!("{key} | {action}") }
                else { format!("{key} | {action} | {}", escape_editor_field(&entry.warning)) }
            }).collect::<Vec<_>>().join("\n");
            entries_fill.buffer().set_text(&contents);
        }
    });

    let tools_save = tools.clone(); let selector_save = selector.clone(); let status_save = status.clone();
    let selected_save = selected_index.clone(); let error_save = error.clone();
    let name_save = name.clone(); let kind_save = kind.clone(); let source_save = source.clone();
    let note_save = note.clone(); let origin_save = origin.clone(); let version_save = version.clone();
    let reviewed_save = reviewed.clone(); let entries_save = entries.clone();
    let update_save = update.clone(); let editor_weak = editor.downgrade();
    save.connect_clicked(move |_| {
        let text_buffer = entries_save.buffer();
        let raw = text_buffer.text(&text_buffer.start_iter(), &text_buffer.end_iter(), true);
        let parsed = raw.lines().filter(|line| !line.trim().is_empty()).map(|line| {
            let (key, action, warning) = parse_editor_entry_line(line)?;
            if key.is_empty() || action.is_empty() {
                return Err("Each entry needs a key or command and an explanation separated by |.".to_string());
            }
            Ok(crate::Shortcut {
                key, action, origin: origin_save.text().to_string(),
                version: version_save.text().to_string(), reviewed: reviewed_save.text().to_string(), warning,
            })
        }).collect::<Result<Vec<_>, String>>();
        let result = parsed.and_then(|entries| {
            let mut tool = Tool {
                name: name_save.text().to_string(), kind: if kind_save.selected() == 1 { "Keys" } else { "Commands" }.into(),
                source: source_save.text().to_string(), note: note_save.text().to_string(), origin: origin_save.text().to_string(),
                version: version_save.text().to_string(), reviewed: reviewed_save.text().to_string(), personal: true, entries,
            };
            let mut combined = tools_save.borrow().clone();
            let selected = selected_save.get();
            let mut existing = combined.clone();
            if let Some(index) = selected { existing.remove(index); }
            tool = catalog::validate_personal_tool(tool, &existing)?;
            if let Some(index) = selected { combined[index] = tool; } else { combined.push(tool); }
            let output = catalog::export_personal_catalog(&combined)?;
            catalog::save_atomic(&catalog::personal_catalog_path(), output.as_bytes())?;
            Ok((combined, selected.unwrap_or_else(|| tools_save.borrow().len())))
        });
        match result {
            Ok((combined, saved_index)) => {
                *tools_save.borrow_mut() = combined;
                let model_names = tool_names(&tools_save);
                let model = gtk::StringList::new(&model_names.iter().map(String::as_str).collect::<Vec<_>>());
                selector_save.set_model(Some(&model)); selector_save.set_selected(saved_index as u32);
                (update_save)(); status_save.set_text("Personal reference saved."); error_save.set_text("");
                if let Some(editor) = editor_weak.upgrade() { editor.set_visible(false); }
            }
            Err(message) => { error_save.set_text(&message); status_save.set_text(&format!("Personal reference not saved: {message}")); }
        }
    });
    let editor_weak = editor.downgrade();
    cancel.connect_clicked(move |_| { if let Some(editor) = editor_weak.upgrade() { editor.set_visible(false); } });
    editor.present();
    PersonalEditorWidgets { window: editor, name, source, note, origin, version, reviewed, entries, choice, save }
}

fn escape_editor_field(value: &str) -> String {
    value.replace('\\', "\\\\").replace('|', "\\|")
}

fn parse_editor_entry_line(line: &str) -> Result<(String, String, String), String> {
    let mut fields = vec![String::new()];
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            if character != '|' && character != '\\' { fields.last_mut().unwrap().push('\\'); }
            fields.last_mut().unwrap().push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '|' && fields.len() < 3 {
            fields.push(String::new());
        } else {
            fields.last_mut().unwrap().push(character);
        }
    }
    if escaped { fields.last_mut().unwrap().push('\\'); }
    if fields.len() < 2 { return Err("Separate each entry's key and explanation with |; escape literal | as \\|.".into()); }
    let mut values = fields.into_iter().map(|value| value.trim().to_string());
    Ok((values.next().unwrap_or_default(), values.next().unwrap_or_default(), values.next().unwrap_or_default()))
}

fn add_editor_row(grid: &gtk::Grid, row: i32, text: &str, widget: &impl IsA<gtk::Widget>) {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0); label.set_mnemonic_widget(Some(widget));
    grid.attach(&label, 0, row, 1, 1);
    widget.set_hexpand(true);
    grid.attach(widget, 1, row, 1, 1);
}

struct SmokeControls {
    window: gtk::ApplicationWindow,
    search: gtk::SearchEntry,
    scope: gtk::DropDown,
    favorites_only: gtk::ToggleButton,
    language_selector: gtk::DropDown,
    scale: gtk::DropDown,
    manage_button: gtk::Button,
    editor_slot: Rc<RefCell<Option<PersonalEditorWidgets>>>,
    info_button: gtk::Button,
    refresh_status: gtk::Button,
    diagnostic: gtk::Label,
    close_dialog: gtk::Button,
    dialog: gtk::Window,
    list: gtk::Grid,
    status: gtk::Label,
    language: Rc<Cell<Language>>,
    tools: Rc<RefCell<Vec<Tool>>>,
}

fn run_gtk_smoke_test(controls: &SmokeControls, scale_path: &std::path::Path) -> bool {
    let mut failures = Vec::new();
    let mut passed = 0usize;
    let mut check = |ok: bool, name: &str| {
        if ok { passed += 1; println!("GTK_TEST PASS: {name}"); } else { eprintln!("GTK_TEST FAIL: {name}"); failures.push(name.to_string()); }
    };
    controls.window.present();
    controls.search.set_text("curl --json");
    controls.search.emit_by_name::<()>("search-changed", &[]);
    controls.scope.set_selected(1);
    drain_gtk_events();
    check(controls.status.text().contains("All-tool results: 1"), "global search updates result count");
    check(widget_contains_text(&controls.list.clone().upcast(), "remote service"), "curated warning is visible in result row");

    let favorite = first_toggle(&controls.list.clone().upcast());
    check(favorite.is_some(), "favorite control is present in accessible GTK widget tree");
    if let Some(favorite) = favorite {
        favorite.set_active(true);
        let saved = catalog::load_favorites().contains(&catalog::favorite_id("curl", "curl --json @data.json <url>"));
        check(saved, "favorite toggle persists to isolated XDG configuration");
        controls.favorites_only.set_active(true);
        drain_gtk_events();
        check(controls.status.text().contains("Favorites: 1"), "favorites-only filter updates results");
        if let Some(favorite) = first_toggle(&controls.list.clone().upcast()) { favorite.set_active(false); }
        controls.favorites_only.set_active(false);
    }

    controls.language_selector.set_selected(Language::Spanish.index());
    check(controls.language.get() == Language::Spanish && controls.search.placeholder_text().as_deref() == Some("Buscar un comando o atajo…"), "language selector localizes controls to Spanish");
    controls.scale.set_selected(1);
    check(controls.window.has_css_class("large-text") && fs::read_to_string(scale_path).is_ok_and(|value| value == "large"), "large text setting applies and persists");
    controls.scale.set_selected(0);
    controls.language_selector.set_selected(Language::English.index());

    controls.info_button.emit_clicked();
    check(controls.dialog.is_visible(), "Info opens as a visible GTK dialog");
    controls.refresh_status.emit_clicked();
    check(controls.diagnostic.text() == controls.language.get().text("isolatedDiagnostic"), "isolated diagnostic refresh confirms no keyboard or permission access");
    controls.close_dialog.emit_clicked();
    check(!controls.dialog.is_visible(), "Info dialog closes from its explicit control");

    let ctrl_f = handle_window_key(&controls.window, &controls.search, gtk::gdk::Key::f, gtk::gdk::ModifierType::CONTROL_MASK);
    check(ctrl_f == glib::Propagation::Stop, "synthetic Ctrl+F is handled by the search-focus path");
    let escape = handle_window_key(&controls.window, &controls.search, gtk::gdk::Key::Escape, gtk::gdk::ModifierType::empty());
    check(escape == glib::Propagation::Stop && !controls.window.is_visible(), "synthetic Escape hides the panel without reading input");
    controls.window.present();

    let empty = catalog::export_personal_catalog(&[]).and_then(|json| {
        catalog::parse_personal_catalog(&json, &[]).map(|references| references.len())
    });
    check(empty == Ok(0), "empty personal export round-trips as valid schema 1 JSON");
    let created = smoke_create_or_edit_personal(controls, false);
    check(created, "GTK personal editor creates a validated local reference");
    let edited = smoke_create_or_edit_personal(controls, true);
    check(edited, "GTK personal editor updates only the explicitly selected reference");
    let fixture = r#"{"schema_version":1,"references":[{"name":"GTK smoke imported","kind":"Commands","source":"https://example.org/docs","note":"Synthetic local test","origin":"GTK test fixture","version":"Not recorded","reviewed":"Not recorded","entries":[{"key":"smoke <target>","action":"Synthetic only","origin":"GTK test fixture","version":"Not recorded","reviewed":"Not recorded","warning":"No command is executed."}]}]}"#;
    let imported = catalog::import_personal_contents(fixture, &mut controls.tools.borrow_mut());
    check(imported == Ok(1) && controls.tools.borrow().iter().any(|tool| tool.name == "GTK smoke imported" && tool.personal), "validated import adds a synthetic personal reference without overwriting");
    let export_path = crate::input::config_path().with_file_name(format!("gtk-smoke-export-{}.json", std::process::id()));
    let exported = catalog::export_personal_catalog(&controls.tools.borrow()).and_then(|json| {
        catalog::save_new(&export_path, json.as_bytes())?;
        catalog::read_personal_file(&export_path)
    });
    check(exported.as_ref().is_ok_and(|json| catalog::parse_personal_catalog(json, &[]).is_ok()), "personal catalog export writes a new file and round-trips");
    let _ = fs::remove_file(export_path);

    drop(check);
    let success = failures.is_empty();
    println!("GTK_TEST SUMMARY: {passed} passed, {} failed", failures.len());
    success
}

fn smoke_create_or_edit_personal(controls: &SmokeControls, edit: bool) -> bool {
    controls.manage_button.emit_clicked();
    drain_gtk_events();
    let Some(editor) = controls.editor_slot.borrow().clone() else { return false; };
    if !editor.window.is_visible() { return false; }
    if edit {
        if editor.choice.model().map_or(0, |model| model.n_items()) < 2 { return false; }
        editor.choice.set_selected(1);
        drain_gtk_events();
        editor.name.set_text("GTK smoke edited");
    } else {
        editor.name.set_text("GTK smoke reference");
    }
    editor.source.set_text("https://example.org/docs");
    editor.note.set_text("Synthetic local test");
    editor.origin.set_text("GTK test fixture");
    editor.version.set_text("Not recorded");
    editor.reviewed.set_text("Not recorded");
    editor.entries.buffer().set_text("smoke <target> | Synthetic only | No command is executed.");
    editor.save.emit_clicked();
    drain_gtk_events();
    if edit {
        controls.tools.borrow().iter().any(|tool| tool.name == "GTK smoke edited" && tool.personal)
            && !controls.tools.borrow().iter().any(|tool| tool.name == "GTK smoke reference")
    } else {
        controls.tools.borrow().iter().any(|tool| tool.name == "GTK smoke reference" && tool.personal)
    }
}

fn drain_gtk_events() {
    let context = glib::MainContext::default();
    for _ in 0..32 { if !context.pending() { break; } context.iteration(false); }
}

fn widget_contains_text(widget: &gtk::Widget, needle: &str) -> bool {
    if widget.clone().downcast::<gtk::Label>().is_ok_and(|label| label.text().contains(needle)) { return true; }
    let mut child = widget.first_child();
    while let Some(current) = child {
        if widget_contains_text(&current, needle) { return true; }
        child = current.next_sibling();
    }
    false
}

fn first_toggle(widget: &gtk::Widget) -> Option<gtk::ToggleButton> {
    if let Ok(toggle) = widget.clone().downcast::<gtk::ToggleButton>() { return Some(toggle); }
    let mut child = widget.first_child();
    while let Some(current) = child {
        if let Some(toggle) = first_toggle(&current) { return Some(toggle); }
        child = current.next_sibling();
    }
    None
}

fn handle_window_key<W: IsA<gtk::Window>>(window: &W, search: &gtk::SearchEntry, key: gtk::gdk::Key, state: gtk::gdk::ModifierType) -> glib::Propagation {
    if key == gtk::gdk::Key::Escape {
        window.as_ref().set_visible(false);
        return glib::Propagation::Stop;
    }
    if key == gtk::gdk::Key::f && state.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
        search.grab_focus(); search.select_region(0, -1);
        return glib::Propagation::Stop;
    }
    glib::Propagation::Proceed
}

pub fn gtk_test_succeeded() -> bool {
    GTK_TEST_SUCCEEDED.load(std::sync::atomic::Ordering::SeqCst)
}

fn reference_basis(tool: &Tool, language: Language) -> &'static str {
    if tool.personal { language.text("personalBasis") }
    else if tool.name == "Ghostty" && tool.origin.starts_with("User Ghostty config") { language.text("ghosttyConfigBasis") }
    else if tool.name == "Ghostty" { language.text("ghosttyDefaultsBasis") }
    else { language.text("archiveStatus") }
}

fn set_source_info(label: &gtk::Label, tool: &Tool, language: Language) {
    label.set_text(&format!(
        "{} · {}\n{}: {}\n{}: {} · {}: {}\n{}: {}\n{}",
        tool.name,
        if tool.kind == "Keys" { language.text("keyboardShortcut") } else { language.text("terminalCommand") },
        language.text("origin"), tool.origin,
        language.text("upstreamVersion"), tool.version,
        language.text("reviewed"), tool.reviewed,
        language.text("source"), tool.source, tool.note
    ));
    let basis = reference_basis(tool, language);
    label.set_text(&format!("{}\n{}", label.text(), basis));
}

fn columns_for_width(width: i32, large_text: bool) -> usize {
    if width >= if large_text { 590 } else { 470 } { 2 } else { 1 }
}

fn cell_position(index: usize, columns: usize) -> (i32, i32) {
    ((index % columns.max(1)) as i32, (index / columns.max(1)) as i32)
}

fn highlight_placeholders(text: &str) -> String {
    let mut output = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('<') {
        let Some(relative_end) = rest[start + 1..].find('>') else { break; };
        let end = start + 1 + relative_end;
        output.push_str(&glib::markup_escape_text(&rest[..start]));
        output.push_str("<span foreground=\"#e8c17a\" weight=\"bold\">");
        output.push_str(&glib::markup_escape_text(&rest[start..=end]));
        output.push_str("</span>");
        rest = &rest[end + 1..];
    }
    output.push_str(&glib::markup_escape_text(rest));
    output
}

pub fn present<W: IsA<gtk::Window>>(window: &W, search: &gtk::SearchEntry) {
    window.present();
    let weak = search.downgrade();
    glib::idle_add_local_once(move || { if let Some(search) = weak.upgrade() { search.grab_focus(); } });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_reflow_between_one_and_two_columns() {
        assert_eq!(cell_position(0, 2), (0, 0));
        assert_eq!(cell_position(1, 2), (1, 0));
        assert_eq!(cell_position(2, 2), (0, 1));
        assert_eq!(cell_position(14, 2), (0, 7));
        assert_eq!(cell_position(1, 1), (0, 1));
        assert_eq!(columns_for_width(460, false), 1);
        assert_eq!(columns_for_width(500, false), 2);
        assert_eq!(columns_for_width(560, true), 1);
        assert_eq!(columns_for_width(620, true), 2);
    }

    #[test]
    fn personal_editor_entry_lines_escape_delimiters_and_backslashes() {
        let line = format!("{} | {} | {}", escape_editor_field("key | value"), escape_editor_field("action\\literal"), escape_editor_field("review | target"));
        assert_eq!(parse_editor_entry_line(&line).unwrap(), ("key | value".into(), "action\\literal".into(), "review | target".into()));
        assert!(parse_editor_entry_line("missing explanation").is_err());
    }

    #[test]
    fn placeholder_markup_escapes_content_and_highlights_variables() {
        assert_eq!(highlight_placeholders("run <target> & <file>"), "run <span foreground=\"#e8c17a\" weight=\"bold\">&lt;target&gt;</span> &amp; <span foreground=\"#e8c17a\" weight=\"bold\">&lt;file&gt;</span>");
        assert_eq!(highlight_placeholders("Ctrl+Shift+C"), "Ctrl+Shift+C");
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
