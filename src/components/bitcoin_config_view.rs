// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

#[derive(Debug, Clone)]
pub struct BitcoinConfigView;

impl BitcoinConfigView {
    pub fn new() -> Self {
        Self
    }

    // Bitcoin Config
    pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
        if app.bitcoin_conf_path.is_some() {
            let items: Vec<ListItem> = app
                .bitcoin_data
                .iter()
                .map(|entry| {
                    let style = if entry.enabled {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let content = Line::from(vec![
                        Span::styled(format!("{} = ", entry.key), style),
                        Span::styled(&entry.value, style),
                        if !entry.enabled {
                            Span::styled(" (disabled)", style)
                        } else {
                            Span::raw("")
                        },
                    ]);

                    ListItem::new(content)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Bitcoin Configuration "),
                )
                .highlight_style(Style::default().bg(Color::Yellow));

            f.render_widget(list, area);
        } else {
            let p = Paragraph::new("Press [Enter] to select a bitcoin.conf file").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Bitcoin Config "),
            );
            f.render_widget(p, area);
        }
    }
}

impl Default for BitcoinConfigView {
    fn default() -> Self {
        Self::new()
    }
}
