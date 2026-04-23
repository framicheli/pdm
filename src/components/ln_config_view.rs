// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug, Clone)]
pub struct LNConfigView;

impl LNConfigView {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    // LN Config
    pub fn render(f: &mut Frame, _app: &mut App, area: Rect) {
        let p = Paragraph::new("LN Config")
            .block(Block::default().borders(Borders::ALL).title(" LN Config "));
        f.render_widget(p, area);
    }
}

impl Default for LNConfigView {
    fn default() -> Self {
        Self::new()
    }
}
