// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug, Clone)]
pub struct SettingsView;

impl SettingsView {
    pub fn new() -> Self {
        Self
    }

    // Settings
    pub fn render(f: &mut Frame, _app: &mut App, area: Rect) {
        let p = Paragraph::new("Settings")
            .block(Block::default().borders(Borders::ALL).title(" Settings "));
        f.render_widget(p, area);
    }
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}
