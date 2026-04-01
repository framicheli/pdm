// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::{App, AppAction};
use crate::bitcoin_config::ConfigEntry;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::Path;

/// Shortens a path to fit within `max_len` characters.
/// Replaces the home directory with `~`, then collapses the middle
/// to `…` if still too long, always keeping the filename visible.
fn shorten_path(path: &Path, max_len: usize) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let full = path.to_string_lossy().into_owned();

    let s = if !home.is_empty() && full.starts_with(&home) {
        format!("~{}", &full[home.len()..])
    } else {
        full
    };

    if s.len() <= max_len {
        return s;
    }

    let p = Path::new(&s);
    let filename = p
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| s.clone());
    let parent_name = p
        .parent()
        .and_then(|p| p.file_name())
        .map(|f| f.to_string_lossy().into_owned());
    let prefix = if s.starts_with('~') { "~" } else { "" };

    // Try ~/…/parent/filename
    if let Some(ref parent) = parent_name {
        let candidate = format!("{}/\u{2026}/{}/{}", prefix, parent, filename);
        if candidate.len() <= max_len {
            return candidate;
        }
    }

    // Try ~/…/filename
    let candidate = format!("{}/\u{2026}/{}", prefix, filename);
    if candidate.len() <= max_len {
        return candidate;
    }

    // Last resort: truncate the right side
    let avail = max_len.saturating_sub(1);
    format!("\u{2026}{}", &s[s.len().saturating_sub(avail)..])
}

#[derive(Debug, Clone)]
pub struct BitcoinConfigView {
    pub selected_index: usize,
    pub editing: bool,
    pub edit_input: String,
    pub save_message: Option<String>,
    pub warning_message: Option<String>,
    pub sidebar_focused: bool,
}

impl BitcoinConfigView {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            editing: false,
            edit_input: String::new(),
            save_message: None,
            warning_message: None,
            sidebar_focused: true,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent, entries: &[ConfigEntry]) -> AppAction {
        self.save_message = None;

        if self.editing {
            match key.code {
                KeyCode::Enter => {
                    let action =
                        AppAction::CommitEdit(self.selected_index, self.edit_input.clone());
                    self.editing = false;
                    self.edit_input.clear();
                    action
                }
                KeyCode::Esc => {
                    self.editing = false;
                    self.edit_input.clear();
                    AppAction::None
                }
                KeyCode::Backspace => {
                    self.edit_input.pop();
                    AppAction::None
                }
                KeyCode::Char(c) => {
                    self.edit_input.push(c);
                    AppAction::None
                }
                _ => AppAction::None,
            }
        } else {
            match key.code {
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    AppAction::None
                }
                KeyCode::Down => {
                    if self.selected_index + 1 < entries.len() {
                        self.selected_index += 1;
                    }
                    AppAction::None
                }
                KeyCode::Enter => {
                    if !entries.is_empty() {
                        self.edit_input = entries[self.selected_index].value.clone();
                        self.editing = true;
                    }
                    AppAction::None
                }
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    AppAction::SaveBitcoinConfig
                }
                KeyCode::Esc => {
                    self.sidebar_focused = true;
                    AppAction::None
                }
                _ => AppAction::None,
            }
        }
    }

    pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
        if app.bitcoin_conf_path.is_none() {
            let p = Paragraph::new("Press [Enter] to select a bitcoin.conf file").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Bitcoin Config "),
            );
            f.render_widget(p, area);
            return;
        }

        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        // Left panel: scrollable entry list
        let items: Vec<ListItem> = app
            .bitcoin_data
            .iter()
            .map(|entry| {
                let label = entry
                    .schema
                    .as_ref()
                    .map(|s| s.description.as_str())
                    .unwrap_or("");

                let (value_display, value_style) = if entry.enabled {
                    (
                        entry.value.clone(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    let placeholder = entry
                        .schema
                        .as_ref()
                        .filter(|s| !s.default.is_empty())
                        .map(|s| format!("default: {}", s.default))
                        .unwrap_or_else(|| "not set".to_string());
                    (
                        format!("({})", placeholder),
                        Style::default().fg(Color::DarkGray),
                    )
                };

                ListItem::new(vec![
                    Line::from(Span::styled(
                        label,
                        Style::default().fg(Color::Gray),
                    )),
                    Line::from(vec![
                        Span::styled(
                            format!("{} = ", entry.key),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(value_display, value_style),
                    ]),
                ])
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(app.bitcoin_config_view.selected_index));

        // " Bitcoin Configuration ---  " = 28 chars fixed, 2 for borders
        const FIXED: usize = 30;
        let path_max = (panels[0].width as usize).saturating_sub(FIXED);
        let title = match &app.bitcoin_conf_path {
            Some(path) => format!(" Bitcoin Configuration --- {} ", shorten_path(path, path_max)),
            None => " Bitcoin Configuration ".to_string(),
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(list, panels[0], &mut list_state);

        // Right panel: detail and edit field
        let right_block = Block::default().borders(Borders::ALL).title(" Detail ");
        let inner = right_block.inner(panels[1]);
        f.render_widget(right_block, panels[1]);

        let selected_entry = app.bitcoin_data.get(app.bitcoin_config_view.selected_index);
        let editing = app.bitcoin_config_view.editing;
        let edit_input = app.bitcoin_config_view.edit_input.clone();

        if let Some(entry) = selected_entry {
            let description = entry
                .schema
                .as_ref()
                .map(|s| s.description.as_str())
                .unwrap_or("Unknown option");
            let type_label = entry
                .schema
                .as_ref()
                .map(|s| format!("{:?}", s.config_type))
                .unwrap_or_default();

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // description
                    Constraint::Length(1), // type
                    Constraint::Length(1), // spacer
                    Constraint::Length(1), // "Value:" label
                    Constraint::Length(3), // value / input box
                    Constraint::Min(0),
                ])
                .split(inner);

            f.render_widget(
                Paragraph::new(description).style(Style::default().fg(Color::White)),
                rows[0],
            );
            f.render_widget(
                Paragraph::new(format!("Type: {}", type_label))
                    .style(Style::default().fg(Color::Gray)),
                rows[1],
            );
            f.render_widget(
                Paragraph::new("Value:").style(Style::default().fg(Color::Gray)),
                rows[3],
            );

            if editing {
                f.render_widget(
                    Paragraph::new(format!("{}_", edit_input))
                        .block(Block::default().borders(Borders::ALL))
                        .style(Style::default().fg(Color::Yellow)),
                    rows[4],
                );
            } else {
                let (display, style) = if entry.enabled {
                    (
                        entry.value.clone(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    let placeholder = entry
                        .schema
                        .as_ref()
                        .filter(|s| !s.default.is_empty())
                        .map(|s| format!("default: {}", s.default))
                        .unwrap_or_else(|| "not set".to_string());
                    (
                        format!("({})", placeholder),
                        Style::default().fg(Color::DarkGray),
                    )
                };
                f.render_widget(
                    Paragraph::new(display)
                        .block(Block::default().borders(Borders::ALL))
                        .style(style),
                    rows[4],
                );
            }
        }
    }
}

impl Default for BitcoinConfigView {
    fn default() -> Self {
        Self::new()
    }
}
