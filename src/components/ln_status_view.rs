// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug, Clone)]
pub struct LNStatusView;

impl LNStatusView {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    // LN Status
    pub fn render(f: &mut Frame, _app: &mut App, area: Rect) {
        let p = Paragraph::new("LN Status")
            .block(Block::default().borders(Borders::ALL).title(" LN Status "));
        f.render_widget(p, area);
    }
}

impl Default for LNStatusView {
    fn default() -> Self {
        Self::new()
    }
}
