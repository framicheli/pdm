// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug, Clone)]
pub struct P2PoolStatusView;

impl P2PoolStatusView {
    pub fn new() -> Self {
        Self
    }

    // P2Pool Status
    pub fn render(f: &mut Frame, _app: &mut App, area: Rect) {
        let p = Paragraph::new("P2Pool Status").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" P2Pool Status "),
        );
        f.render_widget(p, area);
    }
}

impl Default for P2PoolStatusView {
    fn default() -> Self {
        Self::new()
    }
}
