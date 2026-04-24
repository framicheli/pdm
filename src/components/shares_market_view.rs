// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug, Clone)]
pub struct SharesMarketView;

impl SharesMarketView {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    // Shares Market
    pub fn render(f: &mut Frame, _app: &mut App, area: Rect) {
        let p = Paragraph::new("Shares Market").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Shares Market "),
        );
        f.render_widget(p, area);
    }
}

impl Default for SharesMarketView {
    fn default() -> Self {
        Self::new()
    }
}
