// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::{App, BITCOIN_STATUS_TABS};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
};

#[derive(Debug, Clone)]
pub struct BitcoinStatusView;

impl BitcoinStatusView {
    #[must_use] 
    pub fn new() -> Self {
        Self
    }

    pub fn render(f: &mut Frame, app: &App, area: Rect) {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Tabs bar
                Constraint::Min(0),    // Tab content
            ])
            .split(area);

        let tabs = Tabs::new(BITCOIN_STATUS_TABS.to_vec())
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .select(app.bitcoin_status_tab)
            .highlight_style(Style::default().bg(Color::Gray).fg(Color::Black));

        f.render_widget(tabs, outer[0]);

        let content_area = outer[1];
        match app.bitcoin_status_tab {
            // Chain Info
            0 => {
                let text = "Chain Info";
                let p = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(p, content_area);
            }
            // System
            1 => {
                let text = "System";
                let p = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(p, content_area);
            }
            // Logs
            2 => {
                let text = "Logs";
                let p = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(p, content_area);
            }
            // Peers
            3 => {
                let text = "Peers";
                let p = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(p, content_area);
            }
            _ => {}
        }
    }
}

impl Default for BitcoinStatusView {
    fn default() -> Self {
        Self::new()
    }
}
