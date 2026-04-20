// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::{App, BitcoinStatusTab};
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

        let tab_labels = BitcoinStatusTab::ALL.map(BitcoinStatusTab::label).to_vec();
        let tabs = Tabs::new(tab_labels)
            .block(Block::default().borders(Borders::ALL).title(" Info "))
            .select(app.bitcoin_status_tab.as_index())
            .highlight_style(Style::default().bg(Color::Gray).fg(Color::Black));

        f.render_widget(tabs, outer[0]);

        let content_area = outer[1];
        match app.bitcoin_status_tab {
            BitcoinStatusTab::ChainInfo => {
                let p = Paragraph::new("Chain Info")
                    .block(Block::default().borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(p, content_area);
            }
            BitcoinStatusTab::System => {
                let p = Paragraph::new("System")
                    .block(Block::default().borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(p, content_area);
            }
            BitcoinStatusTab::Logs => {
                let p = Paragraph::new("Logs")
                    .block(Block::default().borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(p, content_area);
            }
            BitcoinStatusTab::Peers => {
                let p = Paragraph::new("Peers")
                    .block(Block::default().borders(Borders::ALL))
                    .wrap(Wrap { trim: true });
                f.render_widget(p, content_area);
            }
        }
    }
}

impl Default for BitcoinStatusView {
    fn default() -> Self {
        Self::new()
    }
}
