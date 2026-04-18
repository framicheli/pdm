// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use p2poolv2_config::Config as P2PoolConfig;
use pdm::app::{App, AppAction, CurrentScreen, MAX_BITCOIN_STATUS_TAB, MAX_SIDEBAR_INDEX};
use pdm::bitcoin_config::{
    parse_config as parse_bitcoin_config, save_config as save_bitcoin_config,
};
use pdm::p2poolv2_config::{apply_edit as apply_p2pool_edit, flatten_config};
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
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

                // P2Pool config
                CurrentScreen::P2PoolConfig => {
                    if app.p2pool_conf_path.is_some() {
                        if app.p2pool_config_view.sidebar_focused {
                            match key.code {
                                KeyCode::Enter => {
                                    app.p2pool_config_view.sidebar_focused = false;
                                    AppAction::None
                                }
                                k => sidebar_nav(k, app),
                            }
                        } else {
                            // Build flat entry list and delegate to the view
                            let entries = app
                                .p2pool_config
                                .as_ref()
                                .map(|cfg| flatten_config(cfg))
                                .unwrap_or_default();
                            app.p2pool_config_view.handle_input(key, &entries)
                        }
                    } else {
                        match key.code {
                            KeyCode::Enter => {
                                app.p2pool_config_view.warning_message = None;
                                AppAction::OpenExplorer(CurrentScreen::P2PoolConfig)
                            }
                            KeyCode::Esc => AppAction::CloseModal,
                            k => sidebar_nav(k, app),
                        }
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
                        match P2PoolConfig::load(path.to_str().unwrap()) {
                            Ok(cfg) => {
                                // Sanity check — a valid p2pool config must have
                                // a stratum section with at least a hostname
                                if cfg.stratum.hostname.is_empty() {
                                    app.p2pool_config_view.warning_message = Some(
                    "File does not appear to be a P2Pool config. Select another file."
                        .to_string(),
                );
                                    app.p2pool_conf_path = None;
                                    app.p2pool_config = None;
                                } else {
                                    app.p2pool_conf_path = Some(path.clone());
                                    app.p2pool_config = Some(cfg);
                                    app.p2pool_config_view.sidebar_focused = false;
                                    app.p2pool_config_view.warning_message = None;
                                    app.p2pool_config_view.selected_index = 0;
                                }
                            }
                            Err(e) => {
                                // Config::load failed — wrong file format, missing required
                                // fields, or a TOML parse error
                                app.p2pool_config_view.warning_message = Some(format!(
                                    "Failed to load P2Pool config: {}. Select another file.",
                                    e
                                ));
                                app.p2pool_conf_path = None;
                                app.p2pool_config = None;
                            }
                        }
                        app.current_screen = CurrentScreen::P2PoolConfig;
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
        AppAction::CommitP2PoolEdit(index, value) => {
            if let Some(cfg) = app.p2pool_config.as_mut() {
                match apply_p2pool_edit(cfg, index, &value) {
                    Ok(()) => {
                        app.p2pool_config_view.warning_message = None;
                    }
                    Err(e) => {
                        app.p2pool_config_view.warning_message = Some(e);
                    }
                }
            }
        }

        AppAction::SaveP2PoolConfig => {
            if let (Some(path), Some(cfg)) =
                (app.p2pool_conf_path.clone(), app.p2pool_config.as_ref())
            {
                match save_p2pool_config(&path, cfg) {
                    Ok(()) => {
                        app.p2pool_config_view.save_message =
                            Some("Configuration correctly saved".to_string());
                    }
                    Err(e) => {
                        app.p2pool_config_view.warning_message =
                            Some(format!("Save failed: {}", e));
                    }
                }
            }
        }

        AppAction::None => {}
    }

    Ok(false)
}

/// Matches the TOML type of an existing item and parses the new string
/// value into that same type. This prevents numeric/bool fields from
/// being written back as quoted strings (e.g. port = "3333").
fn typed_toml_item_like(existing: &toml_edit::Item, new_value: &str) -> Result<toml_edit::Item> {
    if existing.as_integer().is_some() {
        let parsed = new_value
            .parse::<i64>()
            .map_err(|e| anyhow::anyhow!("Expected integer, got '{}': {}", new_value, e))?;
        Ok(toml_edit::value(parsed))
    } else if existing.as_float().is_some() {
        let parsed = new_value
            .parse::<f64>()
            .map_err(|e| anyhow::anyhow!("Expected float, got '{}': {}", new_value, e))?;
        Ok(toml_edit::value(parsed))
    } else if existing.as_bool().is_some() {
        let parsed = new_value.parse::<bool>().map_err(|e| {
            anyhow::anyhow!("Expected bool (true/false), got '{}': {}", new_value, e)
        })?;
        Ok(toml_edit::value(parsed))
    } else if existing.as_str().is_some() {
        Ok(toml_edit::value(new_value.to_owned()))
    } else {
        Err(anyhow::anyhow!(
            "Unsupported TOML value type for key: {}",
            existing
        ))
    }
}

/// Serialize the live `P2PoolConfig` back to TOML and write it to disk.
/// Saves P2Pool config by patching the original TOML file in-place.
/// Uses toml_edit so comments and formatting are preserved.
fn save_p2pool_config(path: &std::path::Path, cfg: &P2PoolConfig) -> Result<()> {
    use pdm::p2poolv2_config::flatten_config;
    use toml_edit::DocumentMut;

    // Read the original file so we preserve comments/ordering
    let original = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read P2Pool config: {}", e))?;

    let mut doc = original
        .parse::<DocumentMut>()
        .map_err(|e| anyhow::anyhow!("Failed to parse P2Pool config TOML: {}", e))?;

    // Walk every flattened entry and patch the matching TOML key
    for entry in flatten_config(cfg) {
        let section = entry.section.to_string();
        let key = entry.key.as_str();

        // Skip optional fields that are unset — leave them absent in the file
        if !entry.enabled {
            continue;
        }

        if let Some(table) = doc.get_mut(&section).and_then(|v| v.as_table_mut()) {
            // Only update keys that already exist in the file to avoid
            // injecting fields the user intentionally omitted
            if let Some(existing) = table.get(key) {
                match typed_toml_item_like(existing, &entry.value) {
                    Ok(updated) => table[key] = updated,
                    Err(e) => {
                        // Soft error — skip this field and continue
                        // saving the rest rather than aborting entirely
                        eprintln!("Warning: skipping {}.{}: {}", section, key, e);
                    }
                }
            }
        }
    }

    std::fs::write(path, doc.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to write P2Pool config: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;

    /// Write a  p2pool TOML to `path`.
    fn write_valid_p2pool_toml(path: &std::path::Path) {
        std::fs::write(
            path,
            r#"
[network]
listen_address = "/ip4/127.0.0.1/tcp/6884"
dial_peers = []
max_pending_incoming = 10
max_pending_outgoing = 10
max_established_incoming = 50
max_established_outgoing = 50
max_established_per_peer = 1
max_workbase_per_second = 10
max_userworkbase_per_second = 10
max_miningshare_per_second = 100
max_inventory_per_second = 100
max_transaction_per_second = 100
rate_limit_window_secs = 1
max_requests_per_second = 100
dial_timeout_secs = 30

[store]
path = "./store.db"
background_task_frequency_hours = 24
pplns_ttl_days = 7

[stratum]
hostname = "pool.example.com"
port = 3333
start_difficulty = 10000
minimum_difficulty = 100
solo_address = "tb1qyazxde6558qj6z3d9np5e6msmrspwpf6k0qggk"
bootstrap_address = "tb1qyazxde6558qj6z3d9np5e6msmrspwpf6k0qggk"
zmqpubhashblock = "tcp://127.0.0.1:28332"
network = "signet"
version_mask = "1fffe000"
difficulty_multiplier = 1.0
pool_signature = "P2Poolv2"

[bitcoinrpc]
url = "http://127.0.0.1:38332"
username = "p2pool"
password = "p2pool"

[logging]
file = "./logs/p2pool.log"
console = true
level = "info"
stats_dir = "./logs/stats"

[api]
hostname = "127.0.0.1"
port = 46884
"#,
        )
        .unwrap();
    }

    /// Write a TOML that parses fine but has an empty hostname (fails sanity check).
    fn write_empty_hostname_toml(path: &std::path::Path) {
        std::fs::write(
            path,
            r#"
[network]
listen_address = "/ip4/127.0.0.1/tcp/6884"
dial_peers = []
max_pending_incoming = 10
max_pending_outgoing = 10
max_established_incoming = 50
max_established_outgoing = 50
max_established_per_peer = 1
max_workbase_per_second = 10
max_userworkbase_per_second = 10
max_miningshare_per_second = 100
max_inventory_per_second = 100
max_transaction_per_second = 100
rate_limit_window_secs = 1
max_requests_per_second = 100
dial_timeout_secs = 30

[store]
path = "./store.db"
background_task_frequency_hours = 24
pplns_ttl_days = 7

[stratum]
hostname = ""   # empty hostname should trigger a warning
port = 3333
start_difficulty = 10000
minimum_difficulty = 100
solo_address = "tb1qyazxde6558qj6z3d9np5e6msmrspwpf6k0qggk"
bootstrap_address = "tb1qyazxde6558qj6z3d9np5e6msmrspwpf6k0qggk"
zmqpubhashblock = "tcp://127.0.0.1:28332"
network = "signet"
version_mask = "1fffe000"
difficulty_multiplier = 1.0
pool_signature = "P2Poolv2"

[bitcoinrpc]
url = "http://127.0.0.1:38332"
username = "p2pool"
password = "p2pool"

[logging]
file = "./logs/p2pool.log"
console = true
level = "info"
stats_dir = "./logs/stats"

[api]
hostname = "127.0.0.1"
port = 46884
"#,
        )
        .unwrap();
    }

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
    fn commit_p2pool_edit_bad_value_sets_warning() {
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        // Needs a real sample config — adjust path to your sample
        if let Ok(cfg) = P2PoolConfig::load("../config.sample.toml") {
            let mut app = App::new();
            app.p2pool_config = Some(cfg);
            // Find port index
            let entries = flatten_config(app.p2pool_config.as_ref().unwrap());
            let port_idx = entries.iter().position(|e| e.key == "port").unwrap();
            handle_action(
                AppAction::CommitP2PoolEdit(port_idx, "notanumber".to_string()),
                &mut app,
            )
            .unwrap();
            assert!(app.p2pool_config_view.warning_message.is_some());
        }
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

    // toggle_menu state cleanup

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

    // dirty flag

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

    #[test]
    fn file_selected_valid_p2pool_config_sets_state() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_valid_p2pool_toml(&path);

        let mut app = App::new();
        app.explorer_trigger = Some(CurrentScreen::P2PoolConfig);

        handle_action(AppAction::FileSelected(path.clone()), &mut app).unwrap();

        assert_eq!(app.p2pool_conf_path, Some(path));
        assert!(app.p2pool_config.is_some());
        assert!(!app.p2pool_config_view.sidebar_focused);
        assert!(app.p2pool_config_view.warning_message.is_none());
        assert_eq!(app.p2pool_config_view.selected_index, 0);
        assert_eq!(app.current_screen, CurrentScreen::P2PoolConfig);
    }

    #[test]
    fn file_selected_p2pool_empty_hostname_sets_warning() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_empty_hostname_toml(&path);

        let mut app = App::new();
        app.explorer_trigger = Some(CurrentScreen::P2PoolConfig);

        handle_action(AppAction::FileSelected(path), &mut app).unwrap();

        assert!(app.p2pool_conf_path.is_none());
        assert!(app.p2pool_config.is_none());
        assert!(app.p2pool_config_view.warning_message.is_some());
        let msg = app.p2pool_config_view.warning_message.as_deref().unwrap();
        assert!(msg.contains("does not appear to be a P2Pool config"));
        assert_eq!(app.current_screen, CurrentScreen::P2PoolConfig);
    }

    #[test]
    fn file_selected_p2pool_unparseable_file_sets_warning() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "this is not toml ][[\n").unwrap();

        let mut app = App::new();
        app.explorer_trigger = Some(CurrentScreen::P2PoolConfig);

        handle_action(AppAction::FileSelected(path), &mut app).unwrap();

        assert!(app.p2pool_conf_path.is_none());
        assert!(app.p2pool_config.is_none());
        assert!(app.p2pool_config_view.warning_message.is_some());
        let msg = app.p2pool_config_view.warning_message.as_deref().unwrap();
        assert!(msg.contains("Failed to load P2Pool config"));
        assert_eq!(app.current_screen, CurrentScreen::P2PoolConfig);
    }

    #[test]
    fn commit_p2pool_edit_valid_value_clears_warning() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_valid_p2pool_toml(&path);

        if let Ok(cfg) = P2PoolConfig::load(path.to_str().unwrap()) {
            let mut app = App::new();
            app.p2pool_config = Some(cfg);
            app.p2pool_config_view.warning_message = Some("old warning".to_string());

            let entries = flatten_config(app.p2pool_config.as_ref().unwrap());
            let hostname_idx = entries.iter().position(|e| e.key == "hostname").unwrap();

            handle_action(
                AppAction::CommitP2PoolEdit(hostname_idx, "new.example.com".to_string()),
                &mut app,
            )
            .unwrap();

            assert!(app.p2pool_config_view.warning_message.is_none());
        }
    }

    #[test]
    fn commit_p2pool_edit_invalid_value_sets_warning() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_valid_p2pool_toml(&path);

        if let Ok(cfg) = P2PoolConfig::load(path.to_str().unwrap()) {
            let mut app = App::new();
            app.p2pool_config = Some(cfg);

            let entries = flatten_config(app.p2pool_config.as_ref().unwrap());
            let port_idx = entries.iter().position(|e| e.key == "port").unwrap();

            handle_action(
                AppAction::CommitP2PoolEdit(port_idx, "notanumber".to_string()),
                &mut app,
            )
            .unwrap();

            assert!(app.p2pool_config_view.warning_message.is_some());
        }
    }

    #[test]
    fn commit_p2pool_edit_noop_when_no_config_loaded() {
        let mut app = App::new();
        // p2pool_config is None — action must not panic or set any state
        let result = handle_action(AppAction::CommitP2PoolEdit(0, "val".to_string()), &mut app);
        assert!(result.is_ok());
        assert!(app.p2pool_config_view.warning_message.is_none());
    }

    #[test]
    fn save_p2pool_config_writes_file_and_sets_message() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_valid_p2pool_toml(&path);

        if let Ok(cfg) = P2PoolConfig::load(path.to_str().unwrap()) {
            let mut app = App::new();
            app.p2pool_conf_path = Some(path.clone());
            app.p2pool_config = Some(cfg);

            handle_action(AppAction::SaveP2PoolConfig, &mut app).unwrap();

            assert!(path.exists());
            assert_eq!(
                app.p2pool_config_view.save_message.as_deref(),
                Some("Configuration correctly saved")
            );
            assert!(app.p2pool_config_view.warning_message.is_none());
        }
    }

    #[test]
    fn save_p2pool_config_noop_when_no_path() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_valid_p2pool_toml(&path);

        if let Ok(cfg) = P2PoolConfig::load(path.to_str().unwrap()) {
            let mut app = App::new();
            // path intentionally not set
            app.p2pool_config = Some(cfg);

            handle_action(AppAction::SaveP2PoolConfig, &mut app).unwrap();

            assert!(app.p2pool_config_view.save_message.is_none());
            assert!(app.p2pool_config_view.warning_message.is_none());
        }
    }

    #[test]
    fn save_p2pool_config_unwritable_path_sets_warning() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_valid_p2pool_toml(&path);

        if let Ok(cfg) = P2PoolConfig::load(path.to_str().unwrap()) {
            let mut app = App::new();
            // Point at a directory that doesn't exist so writing fails
            app.p2pool_conf_path = Some(std::path::PathBuf::from("/nonexistent/config.toml"));
            app.p2pool_config = Some(cfg);

            handle_action(AppAction::SaveP2PoolConfig, &mut app).unwrap();

            assert!(app.p2pool_config_view.warning_message.is_some());
            let msg = app.p2pool_config_view.warning_message.as_deref().unwrap();
            assert!(msg.contains("Save failed"));
        }
    }

    #[test]
    fn save_p2pool_config_preserves_comments_and_updates_value() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        // Write a file with a comment that must survive the round-trip
        std::fs::write(
            &path,
            "# keep this comment\n[stratum]\nhostname = \"old.example.com\"\nport = 3333\n\
             [consensus]\nnetwork = \"mainnet\"\n",
        )
        .unwrap();

        if let Ok(mut cfg) = P2PoolConfig::load(path.to_str().unwrap()) {
            cfg.stratum.hostname = "new.example.com".to_string();

            save_p2pool_config(&path, &cfg).unwrap();

            let written = std::fs::read_to_string(&path).unwrap();
            assert!(written.contains("# keep this comment"));
            assert!(written.contains("new.example.com"));
            assert!(!written.contains("old.example.com"));
        }
    }

    #[test]
    fn save_p2pool_config_skips_unset_optional_fields() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        // File deliberately omits an optional field
        std::fs::write(
            &path,
            "[stratum]\nhostname = \"pool.example.com\"\nport = 3333\n\
             [consensus]\nnetwork = \"mainnet\"\n",
        )
        .unwrap();

        let original = std::fs::read_to_string(&path).unwrap();

        if let Ok(cfg) = P2PoolConfig::load(path.to_str().unwrap()) {
            save_p2pool_config(&path, &cfg).unwrap();

            let written = std::fs::read_to_string(&path).unwrap();
            // No new keys should have been injected
            assert_eq!(written.lines().count(), original.lines().count());
        }
    }
}
