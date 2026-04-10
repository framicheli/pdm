// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use p2poolv2_config::Config as P2PoolConfig;
use pdm::app::{App, AppAction, CurrentScreen, MAX_BITCOIN_STATUS_TAB, MAX_SIDEBAR_INDEX};
use pdm::bitcoin_config::{
    parse_config as parse_bitcoin_config, save_config as save_bitcoin_config,
};
use pdm::settings::{load_settings, save_settings};
use pdm::ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::Backend, backend::CrosstermBackend};
use std::io;

fn main() -> Result<()> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run App
    let mut app = App::new();
    app.settings = load_settings();
    bootstrap_from_settings(&mut app);
    let res = run_app(&mut terminal, &mut app);

    // Restore Terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn sidebar_nav(key: KeyCode, app: &mut App) -> AppAction {
    match key {
        KeyCode::Up if app.sidebar_index > 0 => {
            app.sidebar_index -= 1;
            AppAction::ToggleMenu
        }
        KeyCode::Down if app.sidebar_index < MAX_SIDEBAR_INDEX => {
            app.sidebar_index += 1;
            AppAction::ToggleMenu
        }
        _ => AppAction::None,
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    <B as Backend>::Error: Send + Sync + 'static,
{
    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Hard exit (always allowed)
            if key.code == KeyCode::Char('q')
                || (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c'))
            {
                return Ok(());
            }

            let action = match app.current_screen {
                CurrentScreen::FileExplorer => app.explorer.handle_input(key),

                CurrentScreen::BitcoinStatus => match key.code {
                    KeyCode::Left => {
                        if app.bitcoin_status_tab > 0 {
                            app.bitcoin_status_tab -= 1;
                        }
                        AppAction::None
                    }
                    KeyCode::Right => {
                        if app.bitcoin_status_tab < MAX_BITCOIN_STATUS_TAB {
                            app.bitcoin_status_tab += 1;
                        }
                        AppAction::None
                    }
                    k => sidebar_nav(k, app),
                },

                CurrentScreen::BitcoinConfig => {
                    if app.bitcoin_conf_path.is_some() {
                        if app.bitcoin_config_view.sidebar_focused {
                            match key.code {
                                KeyCode::Enter => {
                                    app.bitcoin_config_view.sidebar_focused = false;
                                    AppAction::None
                                }
                                k => sidebar_nav(k, app),
                            }
                        } else {
                            let entries = &app.bitcoin_data;
                            app.bitcoin_config_view.handle_input(key, entries)
                        }
                    } else {
                        match key.code {
                            KeyCode::Enter => {
                                app.bitcoin_config_view.warning_message = None;
                                AppAction::OpenExplorer(CurrentScreen::BitcoinConfig)
                            }
                            KeyCode::Esc => AppAction::CloseModal,
                            k => sidebar_nav(k, app),
                        }
                    }
                }

                CurrentScreen::Settings => {
                    if app.settings_view.sidebar_focused {
                        match key.code {
                            KeyCode::Up => {
                                if app.sidebar_index > 0 {
                                    app.sidebar_index -= 1;
                                    AppAction::ToggleMenu
                                } else {
                                    AppAction::None
                                }
                            }
                            KeyCode::Down => {
                                if app.sidebar_index < 8 {
                                    app.sidebar_index += 1;
                                    AppAction::ToggleMenu
                                } else {
                                    AppAction::None
                                }
                            }
                            KeyCode::Enter => {
                                app.settings_view.sidebar_focused = false;
                                AppAction::None
                            }
                            _ => AppAction::None,
                        }
                    } else {
                        app.settings_view.handle_input(key)
                    }
                }

                _ => match key.code {
                    KeyCode::Enter => {
                        if matches!(app.current_screen, CurrentScreen::P2PoolConfig) {
                            AppAction::OpenExplorer(app.current_screen)
                        } else {
                            AppAction::None
                        }
                    }
                    k => sidebar_nav(k, app),
                },
            };

            if handle_action(action, app)? {
                return Ok(());
            }
        }
    }
}

/// Pre-populate app state from `app.settings`. Called once at startup after
/// settings have been loaded into `app.settings = load_settings()`.
fn bootstrap_from_settings(app: &mut App) {
    // Bitcoin config
    if let Some(path) = app.settings.bitcoin_conf_path.clone() {
        let entries = parse_bitcoin_config(&path).unwrap_or_default();
        if entries.iter().any(|e| e.enabled && e.schema.is_some()) {
            app.bitcoin_conf_path = Some(path);
            app.bitcoin_data = entries;
        }
    }

    // P2Pool config
    if let Some(path) = app.settings.p2pool_conf_path.clone() {
        app.p2pool_conf_path = Some(path.clone());
        if let Ok(cfg) = P2PoolConfig::load(path.to_str().unwrap_or("")) {
            app.p2pool_config = Some(cfg);
        }
    }
}

// Logic Handler
fn handle_action(action: AppAction, app: &mut App) -> Result<bool> {
    match action {
        AppAction::Quit => return Ok(true),

        AppAction::ToggleMenu => app.toggle_menu(),

        AppAction::OpenExplorer(trigger) => {
            app.explorer_trigger = Some(trigger);
            app.current_screen = CurrentScreen::FileExplorer;
        }

        AppAction::CloseModal => {
            app.explorer_trigger = None;
            app.toggle_menu();
        }

        AppAction::FileSelected(path) => {
            if let Some(trigger) = &app.explorer_trigger {
                match trigger {
                    CurrentScreen::P2PoolConfig => {
                        app.p2pool_conf_path = Some(path.clone());
                        if let Ok(cfg) = P2PoolConfig::load(path.to_str().unwrap()) {
                            app.p2pool_config = Some(cfg);
                        }
                        app.current_screen = CurrentScreen::P2PoolConfig;
                        app.settings.p2pool_conf_path = Some(path.clone());
                        let _ = save_settings(&app.settings);
                    }
                    CurrentScreen::BitcoinConfig => match parse_bitcoin_config(&path) {
                        Ok(entries) => {
                            const MIN_KNOWN_KEYS: usize = 1;
                            let known_key_count = entries
                                .iter()
                                .filter(|e| e.enabled && e.schema.is_some())
                                .count();

                            if known_key_count >= MIN_KNOWN_KEYS {
                                app.bitcoin_conf_path = Some(path.clone());
                                app.bitcoin_data = entries;
                                app.bitcoin_config_view.selected_index = 0;
                                app.bitcoin_config_view.dirty = false;
                                app.current_screen = CurrentScreen::BitcoinConfig;
                                app.bitcoin_config_view.sidebar_focused = false;
                                app.bitcoin_config_view.warning_message = None;
                                app.settings.bitcoin_conf_path = Some(path.clone());
                                let _ = save_settings(&app.settings);
                            } else {
                                app.bitcoin_config_view.warning_message = Some(
                                    "File does not appear to be a Bitcoin config. Select another file."
                                        .to_string(),
                                );
                                app.current_screen = CurrentScreen::BitcoinConfig;
                            }
                        }
                        Err(e) => {
                            app.bitcoin_config_view.warning_message = Some(format!(
                                "Failed to read config: {e}. Check permissions and try again."
                            ));
                            app.current_screen = CurrentScreen::BitcoinConfig;
                        }
                    },
                    _ => {}
                }
            }
            app.explorer_trigger = None;
        }

        AppAction::Navigate(screen) => {
            app.current_screen = screen;
        }

        AppAction::CommitEdit(index, value) => {
            if index < app.bitcoin_data.len() {
                app.bitcoin_data[index].value = value;
                app.bitcoin_data[index].enabled = true;
                app.bitcoin_config_view.dirty = true;
            }
        }

        AppAction::SaveBitcoinConfig => {
            if let Some(path) = app.bitcoin_conf_path.clone() {
                save_bitcoin_config(&path, &app.bitcoin_data)?;
                app.bitcoin_config_view.save_message =
                    Some("Configuration correctly saved".to_string());
                app.bitcoin_config_view.dirty = false;
            }
        }

        AppAction::BeginSettingsEdit(index) => {
            // Pre-fill edit_input with the current field value
            let current = match index {
                0 => app
                    .settings
                    .bitcoin_conf_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned()),
                1 => app
                    .settings
                    .p2pool_conf_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned()),
                2 => app
                    .settings
                    .ln_conf_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned()),
                3 => app
                    .settings
                    .shares_market_conf_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned()),
                4 => app
                    .settings
                    .settings_dir_override
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned()),
                _ => None,
            };
            app.settings_view.edit_input = current.unwrap_or_default();
            app.settings_view.editing = true;
        }

        AppAction::CommitSettingsEdit(index, value) => {
            use std::path::PathBuf;
            let path = if value.is_empty() {
                None
            } else {
                Some(PathBuf::from(&value))
            };
            match index {
                0 => app.settings.bitcoin_conf_path = path,
                1 => app.settings.p2pool_conf_path = path,
                2 => app.settings.ln_conf_path = path,
                3 => app.settings.shares_market_conf_path = path,
                4 => app.settings.settings_dir_override = path,
                _ => {}
            }
        }

        AppAction::SaveSettings => {
            save_settings(&app.settings)?;
            app.settings_view.save_message = Some("Settings saved".to_string());
        }

        AppAction::SidebarFocus => {
            // Nothing to do here; the key handler in run_app navigates the sidebar directly.
            // This action is a no-op at the handle_action level.
        }

        AppAction::None => {}
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_app_integration_smoke_test() {
        let backend = TestBackend::new(80, 25);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();

        // Initial render
        terminal.draw(|f| ui::ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!("home_screen", terminal.backend());

        // Simulate sidebar move
        app.sidebar_index = 1;
        app.toggle_menu();

        terminal.draw(|f| ui::ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!("menu_toggled", terminal.backend());

        assert_eq!(app.current_screen, CurrentScreen::BitcoinConfig);
    }

    #[test]
    fn test_file_explorer_flow_state_only() {
        let backend = TestBackend::new(80, 25);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();

        // Navigate to Bitcoin config
        app.sidebar_index = 1;
        app.toggle_menu();
        assert_eq!(app.current_screen, CurrentScreen::BitcoinConfig);

        // Open explorer
        handle_action(
            AppAction::OpenExplorer(CurrentScreen::BitcoinConfig),
            &mut app,
        )
        .unwrap();

        assert_eq!(app.current_screen, CurrentScreen::FileExplorer);

        // Close explorer
        handle_action(AppAction::CloseModal, &mut app).unwrap();
        assert_eq!(app.current_screen, CurrentScreen::BitcoinConfig);

        terminal.draw(|f| ui::ui(f, &mut app)).unwrap();
    }

    #[test]
    fn test_file_explorer_wrap_and_select_sets_config() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        use tempfile::tempdir;

        // Create isolated temporary directory
        let dir = tempdir().unwrap();
        let base = dir.path();

        // Create a fake bitcoin.conf file
        let file_path = base.join("bitcoin.conf");
        std::fs::write(&file_path, "rpcuser=test\n").unwrap();

        let backend = TestBackend::new(80, 25);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new();

        app.explorer.current_dir = base.to_path_buf();
        app.explorer.load_directory();

        handle_action(
            AppAction::OpenExplorer(CurrentScreen::BitcoinConfig),
            &mut app,
        )
        .unwrap();

        // Move selection DOWN to the actual file (skip "..")
        app.explorer
            .handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));

        let action = app
            .explorer
            .handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));

        handle_action(action, &mut app).unwrap();

        assert_eq!(app.bitcoin_conf_path, Some(file_path));

        terminal.draw(|f| ui::ui(f, &mut app)).unwrap();
    }

    #[test]
    fn app_action_open_explorer_sets_state() {
        let mut app = App::new();

        let exited = handle_action(
            AppAction::OpenExplorer(CurrentScreen::BitcoinConfig),
            &mut app,
        )
        .unwrap();

        assert!(!exited);
        assert_eq!(app.current_screen, CurrentScreen::FileExplorer);
        assert_eq!(app.explorer_trigger, Some(CurrentScreen::BitcoinConfig));
    }

    #[test]
    fn app_action_close_modal_returns_to_sidebar() {
        let mut app = App::new();

        app.sidebar_index = 1; // Bitcoin Config
        app.explorer_trigger = Some(CurrentScreen::BitcoinConfig);
        app.current_screen = CurrentScreen::FileExplorer;

        let exited = handle_action(AppAction::CloseModal, &mut app).unwrap();

        assert!(!exited);
        assert_eq!(app.current_screen, CurrentScreen::BitcoinConfig);
        assert!(app.explorer_trigger.is_none());
    }

    #[test]
    fn app_action_quit_requests_exit() {
        let mut app = App::new();

        let exited = handle_action(AppAction::Quit, &mut app).unwrap();

        assert!(exited);
    }

    #[test]
    fn commit_edit_updates_entry_value_and_enables_it() {
        use pdm::bitcoin_config::ConfigEntry;

        let mut app = App::new();
        app.bitcoin_data = vec![
            ConfigEntry {
                key: "rpcuser".to_string(),
                value: "old".to_string(),
                enabled: false,
                schema: None,
                section: None,
            },
            ConfigEntry {
                key: "server".to_string(),
                value: "0".to_string(),
                enabled: true,
                schema: None,
                section: None,
            },
        ];

        handle_action(AppAction::CommitEdit(0, "alice".to_string()), &mut app).unwrap();

        assert_eq!(app.bitcoin_data[0].value, "alice");
        assert!(app.bitcoin_data[0].enabled);
        // Other entries unchanged
        assert_eq!(app.bitcoin_data[1].value, "0");
    }

    #[test]
    fn commit_edit_out_of_bounds_is_noop() {
        let mut app = App::new();
        // bitcoin_data is empty
        let result = handle_action(AppAction::CommitEdit(5, "val".to_string()), &mut app);
        assert!(result.is_ok());
    }

    #[test]
    fn save_bitcoin_config_writes_file_and_sets_message() {
        use pdm::bitcoin_config::ConfigEntry;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("bitcoin.conf");

        let mut app = App::new();
        app.bitcoin_conf_path = Some(path.clone());
        app.bitcoin_data = vec![ConfigEntry {
            key: "rpcuser".to_string(),
            value: "testuser".to_string(),
            enabled: true,
            schema: None,
            section: None,
        }];

        handle_action(AppAction::SaveBitcoinConfig, &mut app).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("rpcuser=testuser"));
        assert_eq!(
            app.bitcoin_config_view.save_message.as_deref(),
            Some("Configuration correctly saved")
        );
    }

    #[test]
    fn save_bitcoin_config_noop_when_no_path() {
        let mut app = App::new();
        // No bitcoin_conf_path set
        let result = handle_action(AppAction::SaveBitcoinConfig, &mut app);
        assert!(result.is_ok());
        assert!(app.bitcoin_config_view.save_message.is_none());
    }

    #[test]
    fn navigate_action_changes_screen() {
        let mut app = App::new();
        handle_action(AppAction::Navigate(CurrentScreen::BitcoinStatus), &mut app).unwrap();
        assert_eq!(app.current_screen, CurrentScreen::BitcoinStatus);
    }

    #[test]
    fn file_selected_invalid_bitcoin_config_sets_warning() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("not_a_config.conf");
        // Write a file with no recognized bitcoin config keys
        std::fs::write(&path, "unknownkey=somevalue\n").unwrap();

        let mut app = App::new();
        app.explorer_trigger = Some(CurrentScreen::BitcoinConfig);

        handle_action(AppAction::FileSelected(path), &mut app).unwrap();

        assert!(app.bitcoin_config_view.warning_message.is_some());
        assert!(app.bitcoin_conf_path.is_none());
        assert_eq!(app.current_screen, CurrentScreen::BitcoinConfig);
    }

    #[test]
    fn bitcoin_config_sidebar_focus_toggle_via_enter() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("bitcoin.conf");
        std::fs::write(&path, "rpcuser=test\n").unwrap();

        let mut app = App::new();
        app.sidebar_index = 1;
        app.toggle_menu();
        handle_action(
            AppAction::OpenExplorer(CurrentScreen::BitcoinConfig),
            &mut app,
        )
        .unwrap();

        app.explorer.current_dir = dir.path().to_path_buf();
        app.explorer.load_directory();

        // Select the file
        app.explorer
            .handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));
        let action = app
            .explorer
            .handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
        handle_action(action, &mut app).unwrap();

        // After file selection, sidebar_focused should be false
        assert!(!app.bitcoin_config_view.sidebar_focused);

        // Pressing Esc via handle_input should set sidebar_focused back
        let entries_clone = app.bitcoin_data.clone();
        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        app.bitcoin_config_view.handle_input(esc, &entries_clone);
        assert!(app.bitcoin_config_view.sidebar_focused);
    }

    // --- toggle_menu state cleanup ---

    #[test]
    fn toggle_menu_clears_bitcoin_config_messages_on_navigate_away() {
        let mut app = App::new();
        app.sidebar_index = 1;
        app.toggle_menu(); // → BitcoinConfig
        app.bitcoin_config_view.warning_message = Some("some warning".to_string());
        app.bitcoin_config_view.save_message = Some("saved".to_string());

        app.sidebar_index = 0;
        app.toggle_menu(); // → Home

        assert!(app.bitcoin_config_view.warning_message.is_none());
        assert!(app.bitcoin_config_view.save_message.is_none());
    }

    #[test]
    fn toggle_menu_cancels_in_progress_edit_on_navigate_away() {
        let mut app = App::new();
        app.sidebar_index = 1;
        app.toggle_menu();
        app.bitcoin_config_view.editing = true;
        app.bitcoin_config_view.edit_input = "draft value".to_string();

        app.sidebar_index = 0;
        app.toggle_menu(); // navigate away

        assert!(!app.bitcoin_config_view.editing);
        assert!(app.bitcoin_config_view.edit_input.is_empty());
    }

    #[test]
    fn toggle_menu_does_not_clear_messages_when_staying_on_other_screen() {
        let mut app = App::new();
        // Start on Home (index 0), set some other state, navigate within Home
        app.sidebar_index = 2;
        app.toggle_menu(); // → BitcoinStatus
        app.bitcoin_config_view.warning_message = Some("keep me".to_string());

        app.sidebar_index = 3;
        app.toggle_menu(); // → P2PoolConfig (never on BitcoinConfig, no clear should happen)

        // Messages only cleared when leaving BitcoinConfig, not from other screens
        assert_eq!(
            app.bitcoin_config_view.warning_message.as_deref(),
            Some("keep me")
        );
    }

    // --- dirty flag ---

    #[test]
    fn commit_edit_sets_dirty_flag() {
        use pdm::bitcoin_config::ConfigEntry;

        let mut app = App::new();
        app.bitcoin_data = vec![ConfigEntry {
            key: "rpcuser".to_string(),
            value: "old".to_string(),
            enabled: true,
            schema: None,
            section: None,
        }];

        handle_action(AppAction::CommitEdit(0, "new".to_string()), &mut app).unwrap();

        assert!(app.bitcoin_config_view.dirty);
        assert_eq!(app.bitcoin_data[0].value, "new");
    }

    #[test]
    fn save_bitcoin_config_clears_dirty_flag() {
        use pdm::bitcoin_config::ConfigEntry;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("bitcoin.conf");

        let mut app = App::new();
        app.bitcoin_conf_path = Some(path.clone());
        app.bitcoin_config_view.dirty = true;
        app.bitcoin_data = vec![ConfigEntry {
            key: "rpcuser".to_string(),
            value: "testuser".to_string(),
            enabled: true,
            schema: None,
            section: None,
        }];

        handle_action(AppAction::SaveBitcoinConfig, &mut app).unwrap();

        assert!(!app.bitcoin_config_view.dirty);
    }

    #[test]
    fn file_selected_resets_dirty_flag() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("bitcoin.conf");
        std::fs::write(&path, "rpcuser=test\n").unwrap();

        let mut app = App::new();
        app.bitcoin_config_view.dirty = true;
        app.explorer_trigger = Some(CurrentScreen::BitcoinConfig);

        handle_action(AppAction::FileSelected(path), &mut app).unwrap();

        assert!(!app.bitcoin_config_view.dirty);
    }

    #[test]
    fn commit_edit_out_of_bounds_does_not_set_dirty() {
        let mut app = App::new();
        // bitcoin_data is empty; CommitEdit with bad index must not set dirty
        handle_action(AppAction::CommitEdit(99, "val".to_string()), &mut app).unwrap();
        assert!(!app.bitcoin_config_view.dirty);
    }

    // --- New settings-related handle_action tests ---

    #[test]
    fn begin_settings_edit_prefills_edit_input_from_current_value() {
        use std::path::PathBuf;

        let mut app = App::new();
        app.settings.bitcoin_conf_path = Some(PathBuf::from("/tmp/bitcoin.conf"));

        handle_action(AppAction::BeginSettingsEdit(0), &mut app).unwrap();

        assert!(app.settings_view.editing);
        assert_eq!(app.settings_view.edit_input, "/tmp/bitcoin.conf");
    }

    #[test]
    fn begin_settings_edit_with_unset_field_prefills_empty() {
        let mut app = App::new();
        // ln_conf_path is None
        handle_action(AppAction::BeginSettingsEdit(2), &mut app).unwrap();
        assert!(app.settings_view.editing);
        assert!(app.settings_view.edit_input.is_empty());
    }

    #[test]
    fn commit_settings_edit_stores_path_in_settings() {
        use std::path::PathBuf;

        let mut app = App::new();
        handle_action(
            AppAction::CommitSettingsEdit(0, "/home/user/bitcoin.conf".to_string()),
            &mut app,
        )
        .unwrap();
        assert_eq!(
            app.settings.bitcoin_conf_path,
            Some(PathBuf::from("/home/user/bitcoin.conf"))
        );
    }

    #[test]
    fn commit_settings_edit_empty_value_clears_path() {
        use std::path::PathBuf;

        let mut app = App::new();
        app.settings.p2pool_conf_path = Some(PathBuf::from("/tmp/p2pool.toml"));

        handle_action(AppAction::CommitSettingsEdit(1, String::new()), &mut app).unwrap();
        assert!(app.settings.p2pool_conf_path.is_none());
    }

    #[test]
    fn commit_settings_edit_all_field_indices() {
        use std::path::PathBuf;

        let mut app = App::new();
        let val = "/tmp/test.conf".to_string();

        for idx in 0..5 {
            handle_action(AppAction::CommitSettingsEdit(idx, val.clone()), &mut app).unwrap();
        }

        assert_eq!(app.settings.bitcoin_conf_path, Some(PathBuf::from(&val)));
        assert_eq!(app.settings.p2pool_conf_path, Some(PathBuf::from(&val)));
        assert_eq!(app.settings.ln_conf_path, Some(PathBuf::from(&val)));
        assert_eq!(
            app.settings.shares_market_conf_path,
            Some(PathBuf::from(&val))
        );
        assert_eq!(
            app.settings.settings_dir_override,
            Some(PathBuf::from(&val))
        );
    }

    #[test]
    fn commit_settings_edit_out_of_bounds_is_noop() {
        let mut app = App::new();
        let result = handle_action(AppAction::CommitSettingsEdit(99, "x".to_string()), &mut app);
        assert!(result.is_ok());
    }

    #[test]
    fn save_settings_action_writes_file_and_sets_message() {
        use std::path::PathBuf;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let mut app = App::new();
        app.settings.bitcoin_conf_path = Some(PathBuf::from("/tmp/bitcoin.conf"));

        let path = dir.path().join("settings.toml");
        let content = toml::to_string_pretty(&app.settings).unwrap();
        std::fs::write(&path, content).unwrap();

        let result = handle_action(AppAction::SaveSettings, &mut app);
        assert!(result.is_ok());
        assert_eq!(
            app.settings_view.save_message.as_deref(),
            Some("Settings saved")
        );
    }

    #[test]
    fn sidebar_focus_action_is_noop() {
        let mut app = App::new();
        app.current_screen = CurrentScreen::Settings;
        let result = handle_action(AppAction::SidebarFocus, &mut app);
        assert!(result.is_ok());
        assert_eq!(app.current_screen, CurrentScreen::Settings);
    }

    #[test]
    fn file_selected_bitcoin_config_persists_to_settings() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("bitcoin.conf");
        std::fs::write(&path, "rpcuser=test\n").unwrap();

        let mut app = App::new();
        app.explorer_trigger = Some(CurrentScreen::BitcoinConfig);

        handle_action(AppAction::FileSelected(path.clone()), &mut app).unwrap();

        assert_eq!(app.settings.bitcoin_conf_path, Some(path));
    }

    #[test]
    fn bootstrap_from_settings_loads_bitcoin_config() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("bitcoin.conf");
        std::fs::write(&path, "rpcuser=test\n").unwrap();

        let mut app = App::new();
        app.settings.bitcoin_conf_path = Some(path.clone());

        bootstrap_from_settings(&mut app);

        assert_eq!(app.bitcoin_conf_path, Some(path));
        assert!(!app.bitcoin_data.is_empty());
    }

    #[test]
    fn bootstrap_from_settings_ignores_invalid_bitcoin_config() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.conf");
        std::fs::write(&path, "notakey=value\n").unwrap();

        let mut app = App::new();
        app.settings.bitcoin_conf_path = Some(path);

        bootstrap_from_settings(&mut app);

        // Invalid config: bitcoin_conf_path must NOT be set on app
        assert!(app.bitcoin_conf_path.is_none());
    }
}
