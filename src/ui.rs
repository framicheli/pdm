// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::{App, CurrentScreen};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};

pub fn ui(f: &mut Frame, app: &mut App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main area
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    let main_row = outer[0];
    let status_bar_area = outer[1];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25), // Sidebar
            Constraint::Min(0),     // Main Content
        ])
        .split(main_row);

    //  Sidebar
    let items = vec![
        ListItem::new("Home"),
        ListItem::new("Bitcoin Config"),
        ListItem::new("Bitcoin Status"),
        ListItem::new("P2Pool Config"),
        ListItem::new("P2Pool Status"),
        ListItem::new("LN Config"),
        ListItem::new("LN Status"),
        ListItem::new("Shares Market"),
    ];

    // Highlight the active one
    let mut state = ListState::default();
    state.select(Some(app.sidebar_index));

    let sidebar = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" PDM "))
        .highlight_style(Style::default().bg(Color::Gray).fg(Color::Black));

    f.render_stateful_widget(sidebar, chunks[0], &mut state);

    // Main Content
    let main_area = chunks[1];

    match app.current_screen {
        CurrentScreen::Home => {
            let p = Paragraph::new("Welcome to PDM.\n\nSelect a config from the sidebar to edit.")
                .block(Block::default().borders(Borders::ALL).title(" Home "))
                .wrap(Wrap { trim: true });
            f.render_widget(p, main_area);
        }
        CurrentScreen::BitcoinConfig => {
            if app.bitcoin_conf_path.is_some() {
                render_bitcoin_view(f, app, main_area);
            } else {
                let p = Paragraph::new("Press [Enter] to select a bitcoin.conf file").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Bitcoin Config "),
                );
                f.render_widget(p, main_area);
            }
        }
        CurrentScreen::BitcoinStatus => {
            render_bitcoin_status_view(f, app, main_area);
        }
        CurrentScreen::P2PoolConfig => {
            if app.p2pool_conf_path.is_some() {
                render_p2pool_view(f, app, main_area);
            } else {
                let p = Paragraph::new("Press [Enter] to select a p2poolv2 config file").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" P2Pool Config "),
                );
                f.render_widget(p, main_area);
            }
        }
        CurrentScreen::P2PoolStatus => {
            let p = Paragraph::new("P2Pool Status").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" P2Pool Status "),
            );
            f.render_widget(p, main_area);
        }
        CurrentScreen::LNConfig => {
            let p = Paragraph::new("LN Config")
                .block(Block::default().borders(Borders::ALL).title(" LN Config "));
            f.render_widget(p, main_area);
        }
        CurrentScreen::LNStatus => {
            let p = Paragraph::new("LN Status")
                .block(Block::default().borders(Borders::ALL).title(" LN Status "));
            f.render_widget(p, main_area);
        }
        CurrentScreen::SharesMarket => {
            let p = Paragraph::new("Share Market").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Share Market "),
            );
            f.render_widget(p, main_area);
        }
        CurrentScreen::FileExplorer => {
            render_file_explorer(f, app, main_area);
        }
        _ => {}
    }

    render_status_bar(f, app, status_bar_area);
}

fn hint(key: &str, desc: &str) -> Vec<Span<'static>> {
    vec![
        Span::styled(
            format!(" {key} "),
            Style::default().bg(Color::DarkGray).fg(Color::White),
        ),
        Span::styled(format!(" {desc}  "), Style::default().fg(Color::DarkGray)),
    ]
}

// Status bar
fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let mut spans: Vec<Span> = Vec::new();

    match app.current_screen {
        CurrentScreen::FileExplorer => {
            spans.extend(hint("↑↓", "Navigate"));
            spans.extend(hint("Enter", "Select"));
            spans.extend(hint("Esc", "Cancel"));
        }
        CurrentScreen::BitcoinConfig if app.bitcoin_conf_path.is_some() => {
            spans.extend(hint("↑↓", "Navigate"));
            spans.extend(hint("Enter", "Open file"));
            spans.extend(hint("q", "Quit"));
        }
        CurrentScreen::P2PoolConfig if app.p2pool_conf_path.is_some() => {
            spans.extend(hint("↑↓", "Navigate"));
            spans.extend(hint("Enter", "Open file"));
            spans.extend(hint("q", "Quit"));
        }
        CurrentScreen::BitcoinStatus => {
            spans.extend(hint("↑↓", "Navigate sidebar"));
            spans.extend(hint("←→", "Switch tab"));
            spans.extend(hint("q", "Quit"));
        }
        _ => {
            spans.extend(hint("↑↓", "Navigate sidebar"));
            spans.extend(hint("Enter", "Select"));
            spans.extend(hint("q", "Quit"));
        }
    }

    let bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black));
    f.render_widget(bar, area);
}

fn render_bitcoin_status_view(f: &mut Frame, app: &App, area: Rect) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Tabs bar
            Constraint::Min(0),    // Tab content
        ])
        .split(area);

    let tabs = Tabs::new(vec!["Chain Info", "System", "Logs", "Peers"])
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

fn render_file_explorer(f: &mut Frame, app: &mut App, area: Rect) {
    let files: Vec<ListItem> = app
        .explorer
        .files
        .iter()
        .map(|path| {
            let display_name = if path.ends_with("..") {
                "📁 ..".to_string()
            } else {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if path.is_dir() {
                    format!("📁 {}", name)
                } else {
                    format!("📄 {}", name)
                }
            };
            ListItem::new(display_name)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.explorer.selected_index));

    let title = format!(" Select File (Current: {:?}) ", app.explorer.current_dir);

    let list = List::new(files)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut state);
}

fn render_p2pool_view(f: &mut Frame, app: &mut App, area: Rect) {
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
}

fn render_bitcoin_view(f: &mut Frame, app: &mut App, area: Rect) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn make_terminal() -> Terminal<TestBackend> {
        Terminal::new(TestBackend::new(80, 24)).unwrap()
    }

    #[test]
    fn test_home_screen_render() {
        let mut terminal = make_terminal();
        let mut app = App::new();
        terminal.draw(|f| ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!(terminal.backend());
    }

    #[test]
    fn test_bitcoin_config_screen_render() {
        let mut terminal = make_terminal();
        let mut app = App::new();
        app.sidebar_index = 1;
        app.toggle_menu();
        terminal.draw(|f| ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!(terminal.backend());
    }

    #[test]
    fn test_bitcoin_status_screen_render() {
        let mut terminal = make_terminal();
        let mut app = App::new();
        app.sidebar_index = 2;
        app.toggle_menu();
        terminal.draw(|f| ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!(terminal.backend());
    }

    #[test]
    fn test_bitcoin_status_tab_system_render() {
        let mut terminal = make_terminal();
        let mut app = App::new();
        app.sidebar_index = 2;
        app.toggle_menu();
        app.bitcoin_status_tab = 1;
        terminal.draw(|f| ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!(terminal.backend());
    }

    #[test]
    fn test_bitcoin_status_tab_logs_render() {
        let mut terminal = make_terminal();
        let mut app = App::new();
        app.sidebar_index = 2;
        app.toggle_menu();
        app.bitcoin_status_tab = 2;
        terminal.draw(|f| ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!(terminal.backend());
    }

    #[test]
    fn test_bitcoin_status_tab_peers_render() {
        let mut terminal = make_terminal();
        let mut app = App::new();
        app.sidebar_index = 2;
        app.toggle_menu();
        app.bitcoin_status_tab = 3;
        terminal.draw(|f| ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!(terminal.backend());
    }

    #[test]
    fn test_p2pool_config_screen_render() {
        let mut terminal = make_terminal();
        let mut app = App::new();
        app.sidebar_index = 3;
        app.toggle_menu();
        terminal.draw(|f| ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!(terminal.backend());
    }

    #[test]
    fn test_p2pool_status_screen_render() {
        let mut terminal = make_terminal();
        let mut app = App::new();
        app.sidebar_index = 4;
        app.toggle_menu();
        terminal.draw(|f| ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!(terminal.backend());
    }

    #[test]
    fn test_ln_config_screen_render() {
        let mut terminal = make_terminal();
        let mut app = App::new();
        app.sidebar_index = 5;
        app.toggle_menu();
        terminal.draw(|f| ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!(terminal.backend());
    }

    #[test]
    fn test_ln_status_screen_render() {
        let mut terminal = make_terminal();
        let mut app = App::new();
        app.sidebar_index = 6;
        app.toggle_menu();
        terminal.draw(|f| ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!(terminal.backend());
    }

    #[test]
    fn test_shares_market_screen_render() {
        let mut terminal = make_terminal();
        let mut app = App::new();
        app.sidebar_index = 7;
        app.toggle_menu();
        terminal.draw(|f| ui(f, &mut app)).unwrap();
        insta::assert_debug_snapshot!(terminal.backend());
    }
}
