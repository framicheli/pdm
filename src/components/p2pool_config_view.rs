// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

#[derive(Debug, Clone)]
pub struct P2PoolConfigView;

impl P2PoolConfigView {
    #[must_use] 
    pub fn new() -> Self {
        Self
    }

    // P2Pool Config
    pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
        if app.p2pool_conf_path.is_some() {
            let mut items: Vec<ListItem> = Vec::new();

            if let Some(cfg) = &app.p2pool_config {
                // STRATUM
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("[stratum] ", Style::default().fg(Color::Blue)),
                    Span::raw(format!("hostname = {}", cfg.stratum.hostname)),
                ])));

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("[stratum] ", Style::default().fg(Color::Blue)),
                    Span::raw(format!("port = {}", cfg.stratum.port)),
                ])));

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("[stratum] ", Style::default().fg(Color::Blue)),
                    Span::raw(format!(
                        "start_difficulty = {}",
                        cfg.stratum.start_difficulty
                    )),
                ])));

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("[stratum] ", Style::default().fg(Color::Blue)),
                    Span::raw(format!(
                        "minimum_difficulty = {}",
                        cfg.stratum.minimum_difficulty
                    )),
                ])));

                // BITCOIN RPC
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("[bitcoinrpc] ", Style::default().fg(Color::Blue)),
                    Span::raw(format!("url = {}", cfg.bitcoinrpc.url)),
                ])));

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("[bitcoinrpc] ", Style::default().fg(Color::Blue)),
                    Span::raw(format!("username = {}", cfg.bitcoinrpc.username)),
                ])));

                // NETWORK
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("[network] ", Style::default().fg(Color::Blue)),
                    Span::raw(format!("listen_address = {}", cfg.network.listen_address)),
                ])));

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("[network] ", Style::default().fg(Color::Blue)),
                    Span::raw(format!(
                        "max_established_incoming = {}",
                        cfg.network.max_established_incoming
                    )),
                ])));

                // STORE
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("[store] ", Style::default().fg(Color::Blue)),
                    Span::raw(format!("path = {}", cfg.store.path)),
                ])));

                // API
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("[api] ", Style::default().fg(Color::Blue)),
                    Span::raw(format!("hostname = {}", cfg.api.hostname)),
                ])));

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("[api] ", Style::default().fg(Color::Blue)),
                    Span::raw(format!("port = {}", cfg.api.port)),
                ])));
            }

            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" P2Pool Configuration "),
            );

            f.render_widget(list, area);
        } else {
            let p = Paragraph::new("Press [Enter] to select a p2poolv2 config file").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" P2Pool Config "),
            );
            f.render_widget(p, area);
        }
    }
}

impl Default for P2PoolConfigView {
    fn default() -> Self {
        Self::new()
    }
}
