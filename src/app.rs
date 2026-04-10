// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::bitcoin_config::ConfigEntry as BitcoinEntry;
use crate::components::bitcoin_config_view::BitcoinConfigView;
use crate::components::file_explorer::FileExplorer;
use crate::components::settings_view::SettingsView;
use crate::settings::Settings;
use p2poolv2_config::Config as P2PoolConfig;
use std::path::PathBuf;

/// Sidebar items labels
pub const SIDEBAR_ITEMS: &[(&str, CurrentScreen)] = &[
    ("Home", CurrentScreen::Home),
    ("Bitcoin Config", CurrentScreen::BitcoinConfig),
    ("Bitcoin Status", CurrentScreen::BitcoinStatus),
    ("P2Pool Config", CurrentScreen::P2PoolConfig),
    ("P2Pool Status", CurrentScreen::P2PoolStatus),
    ("LN Config", CurrentScreen::LNConfig),
    ("LN Status", CurrentScreen::LNStatus),
    ("Shares Market", CurrentScreen::SharesMarket),
    ("Settings", CurrentScreen::Settings),
];

pub const MAX_SIDEBAR_INDEX: usize = SIDEBAR_ITEMS.len() - 1;

/// Tab labels for the Bitcoin Status view
pub const BITCOIN_STATUS_TABS: &[&str] = &["Chain Info", "System", "Logs", "Peers"];

pub const MAX_BITCOIN_STATUS_TAB: usize = BITCOIN_STATUS_TABS.len() - 1;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CurrentScreen {
    Home,
    BitcoinConfig,
    BitcoinStatus,
    P2PoolConfig,
    P2PoolStatus,
    LNConfig,
    LNStatus,
    SharesMarket,
    FileExplorer,
    Settings,
}

/// Actions that components (Explorer, Editors) can trigger.
/// This decouples input handling from business logic.
#[derive(Debug, Clone)]
pub enum AppAction {
    None,
    Quit,
    ToggleMenu,
    Navigate(CurrentScreen),
    // Triggers the file explorer for a specific screen
    OpenExplorer(CurrentScreen),
    // Returned by the Explorer when user picks a file
    FileSelected(PathBuf),
    // Closes the explorer without selection
    CloseModal,
    // Commits an edited value: (entry index, new value)
    CommitEdit(usize, String),
    // Saves bitcoin config to disk
    SaveBitcoinConfig,
    // Commits a settings field edit: (field index, new value string)
    CommitSettingsEdit(usize, String),
    // Saves settings to disk
    SaveSettings,
    // Begin editing a settings field (pre-fills edit_input from current value)
    BeginSettingsEdit(usize),
    // Return focus to sidebar from any content view
    SidebarFocus,
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub sidebar_index: usize,
    pub explorer_trigger: Option<CurrentScreen>,
    pub bitcoin_conf_path: Option<PathBuf>,
    pub p2pool_conf_path: Option<PathBuf>,
    pub explorer: FileExplorer,
    pub bitcoin_config_view: BitcoinConfigView,
    pub settings_view: SettingsView,
    pub p2pool_config: Option<P2PoolConfig>,
    pub bitcoin_data: Vec<BitcoinEntry>,
    pub bitcoin_status_tab: usize,
    pub settings: Settings,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Home,
            sidebar_index: 0,
            explorer_trigger: None,
            bitcoin_conf_path: None,
            p2pool_conf_path: None,
            explorer: FileExplorer::new(),
            bitcoin_config_view: BitcoinConfigView::new(),
            settings_view: SettingsView::new(),
            p2pool_config: None,
            bitcoin_data: Vec::new(),
            bitcoin_status_tab: 0,
            settings: Settings::default(),
        }
    }

    // Logic to switch between sidebar items
    pub fn toggle_menu(&mut self) {
        if self.current_screen == CurrentScreen::BitcoinConfig {
            self.bitcoin_config_view.warning_message = None;
            self.bitcoin_config_view.save_message = None;
            self.bitcoin_config_view.editing = false;
            self.bitcoin_config_view.edit_input.clear();
        }
        if let Some(&(_, screen)) = SIDEBAR_ITEMS.get(self.sidebar_index) {
            self.current_screen = screen;
        }
    }
}
impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
