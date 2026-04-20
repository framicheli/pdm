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

/// The active tab in the Bitcoin Status view.
/// Adding a variant here automatically causes the `match` in
/// `bitcoin_status_view.rs` to fail to compile until a new arm is added.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BitcoinStatusTab {
    #[default]
    ChainInfo,
    System,
    Logs,
    Peers,
}

impl BitcoinStatusTab {
    /// All tabs in display order — used to build the `Tabs` widget labels.
    pub const ALL: [Self; 4] = [Self::ChainInfo, Self::System, Self::Logs, Self::Peers];

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::ChainInfo => "Chain Info",
            Self::System => "System",
            Self::Logs => "Logs",
            Self::Peers => "Peers",
        }
    }

    /// Index into `ALL`, used by the ratatui `Tabs` widget's `.select()` call.
    #[must_use]
    pub fn as_index(self) -> usize {
        match self {
            Self::ChainInfo => 0,
            Self::System => 1,
            Self::Logs => 2,
            Self::Peers => 3,
        }
    }

    /// Move one tab left, clamped at the first tab.
    #[must_use]
    pub fn prev(self) -> Self {
        match self {
            Self::ChainInfo | Self::System => Self::ChainInfo,
            Self::Logs => Self::System,
            Self::Peers => Self::Logs,
        }
    }

    /// Move one tab right, clamped at the last tab.
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::ChainInfo => Self::System,
            Self::System => Self::Logs,
            Self::Logs | Self::Peers => Self::Peers,
        }
    }
}

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

/// Identifies which screen (and optionally which field) triggered the file explorer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplorerTrigger {
    BitcoinConfig,
    P2PoolConfig,
    /// The `usize` is the settings field index (0–`FIELD_COUNT - 1`).
    Settings(usize),
}

/// Actions that components (Explorer, Editors) can trigger.
/// This decouples input handling from business logic.
#[derive(Debug, Clone)]
pub enum AppAction {
    None,
    Quit,
    ToggleMenu,
    Navigate(CurrentScreen),
    // Triggers the file explorer; the trigger identifies the caller
    OpenExplorer(ExplorerTrigger),
    // Returned by the Explorer when user picks a file
    FileSelected(PathBuf),
    // Closes the explorer without selection
    CloseModal,
    // Commits an edited value: (entry index, new value)
    CommitEdit(usize, String),
    // Saves bitcoin config to disk
    SaveBitcoinConfig,
    // Open the file explorer to pick a path for a settings field (field index)
    OpenExplorerForSettings(usize),
    // Clear a settings field by index, setting it back to None
    ClearSettingsField(usize),
}

pub struct App {
    pub current_screen: CurrentScreen,
    pub sidebar_index: usize,
    pub explorer_trigger: Option<ExplorerTrigger>,
    pub bitcoin_conf_path: Option<PathBuf>,
    pub p2pool_conf_path: Option<PathBuf>,
    pub explorer: FileExplorer,
    pub bitcoin_config_view: BitcoinConfigView,
    pub settings_view: SettingsView,
    pub p2pool_config: Option<P2PoolConfig>,
    pub bitcoin_data: Vec<BitcoinEntry>,
    pub bitcoin_status_tab: BitcoinStatusTab,
    pub settings: Settings,
    /// Cached value of the `HOME` environment variable, used for path display.
    /// Populated once at startup to avoid repeated syscalls during rendering.
    pub home_dir: String,
    /// Cached result of `settings::config_dir()`, used to display the default
    /// settings storage path without repeated env-var lookups during rendering.
    pub config_dir: PathBuf,
}

impl App {
    #[must_use]
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
            bitcoin_status_tab: BitcoinStatusTab::default(),
            settings: Settings::default(),
            home_dir: std::env::var("HOME").unwrap_or_default(),
            config_dir: crate::settings::config_dir().unwrap_or_default(),
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
