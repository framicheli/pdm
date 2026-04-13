// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::{App, AppAction};
use crate::bitcoin_config::ConfigEntry;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::path::Path;

/// Shortens a path to fit within `max_len` Unicode scalar values (terminal columns)
fn shorten_path(path: &Path, max_len: usize) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let full = path.to_string_lossy().into_owned();

    let s = if !home.is_empty() && full.starts_with(&home) {
        format!("~{}", full.strip_prefix(&home).unwrap_or(&full))
    } else {
        full
    };

    if s.chars().count() <= max_len {
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
        if candidate.chars().count() <= max_len {
            return candidate;
        }
    }

    // Try ~/…/filename
    let candidate = format!("{}/\u{2026}/{}", prefix, filename);
    if candidate.chars().count() <= max_len {
        return candidate;
    }

    // Truncate the right side on character boundaries
    let avail = max_len.saturating_sub(1);
    let total_chars = s.chars().count();
    let suffix: String = s.chars().skip(total_chars.saturating_sub(avail)).collect();
    format!("\u{2026}{}", suffix)
}

#[derive(Debug, Clone)]
pub struct BitcoinConfigView {
    pub selected_index: usize,
    pub editing: bool,
    pub edit_input: String,
    pub save_message: Option<String>,
    pub warning_message: Option<String>,
    pub sidebar_focused: bool,
    /// True when entries have been committed (via CommitEdit) but not yet saved to disk.
    pub dirty: bool,
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
            dirty: false,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent, entries: &[ConfigEntry]) -> AppAction {
        if self.editing {
            match key.code {
                KeyCode::Enter => {
                    let action =
                        AppAction::CommitEdit(self.selected_index, self.edit_input.clone());
                    self.editing = false;
                    self.edit_input.clear();
                    self.save_message = None;
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
                    self.save_message = None;
                    AppAction::None
                }
                KeyCode::Down => {
                    if self.selected_index + 1 < entries.len() {
                        self.selected_index += 1;
                    }
                    self.save_message = None;
                    AppAction::None
                }
                KeyCode::Enter => {
                    if !entries.is_empty() {
                        self.edit_input = entries[self.selected_index].value.clone();
                        self.editing = true;
                        self.save_message = None;
                    }
                    AppAction::None
                }
                KeyCode::Char('s') => AppAction::SaveBitcoinConfig,
                KeyCode::Esc => {
                    self.sidebar_focused = true;
                    self.save_message = None;
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
                    Line::from(Span::styled(label, Style::default().fg(Color::Gray))),
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

        // Border style: dim both panels when the user is navigating the main sidebar
        let panel_style = if app.bitcoin_config_view.sidebar_focused {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let dirty = app.bitcoin_config_view.dirty;
        const FIXED: usize = 33;
        let path_max = (panels[0].width as usize).saturating_sub(FIXED);
        let title = match &app.bitcoin_conf_path {
            Some(path) => format!(
                " {}Bitcoin Configuration --- {} ",
                if dirty { "● " } else { "" },
                shorten_path(path, path_max)
            ),
            None => " Bitcoin Configuration ".to_string(),
        };
        let title_style = if dirty {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .title_style(title_style)
                    .border_style(panel_style),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(list, panels[0], &mut list_state);

        // Right panel: detail and edit field
        let right_block = Block::default()
            .borders(Borders::ALL)
            .title(" Detail ")
            .border_style(panel_style);
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
                .map(|s| format!("{}", s.config_type))
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
                    Paragraph::new(edit_input.as_str())
                        .block(Block::default().borders(Borders::ALL))
                        .style(Style::default().fg(Color::Yellow)),
                    rows[4],
                );
                let cursor_x = (rows[4].x + 1 + edit_input.chars().count() as u16)
                    .min(rows[4].x + rows[4].width.saturating_sub(2));
                let cursor_y = rows[4].y + 1;
                f.set_cursor_position((cursor_x, cursor_y));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppAction;
    use crate::bitcoin_config::ConfigEntry;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn entry(key: &str, value: &str, enabled: bool) -> ConfigEntry {
        ConfigEntry {
            key: key.to_string(),
            value: value.to_string(),
            enabled,
            schema: None,
            section: None,
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    // --- shorten path ---

    #[test]
    fn shorten_path_short_enough_unchanged() {
        let p = Path::new("/foo/bar.conf");
        assert_eq!(shorten_path(p, 100), "/foo/bar.conf");
    }

    #[test]
    fn shorten_path_collapses_to_parent_filename() {
        // Path with no HOME prefix, long enough to trigger collapse
        let p = Path::new("/a/very/long/path/to/parent/file.conf");
        let result = shorten_path(p, 20);
        assert!(result.contains("file.conf"));
        assert!(result.chars().count() <= 20);
    }

    #[test]
    fn shorten_path_collapses_to_filename_only() {
        // Parent/filename still too long → ~/…/filename
        let long_parent = "/a/b/c/d/longlonglonglongparent/file.conf";
        let p = Path::new(long_parent);
        let result = shorten_path(p, 18);
        assert!(result.contains("file.conf"));
        assert!(result.chars().count() <= 18);
    }

    #[test]
    fn shorten_path_last_resort_truncation() {
        // Even filename alone doesn't fit → truncate with ellipsis
        let p = Path::new("/a/b/c/d/e/verylongfilename.conf");
        let result = shorten_path(p, 5);
        assert!(result.starts_with('\u{2026}'));
        assert!(result.chars().count() <= 5);
    }

    #[test]
    fn shorten_path_multibyte_chars_respected() {
        // Each of these chars is 3 bytes but 1 column; byte-length checks would fail here
        let p = Path::new("/日本語/パス/ファイル.conf");
        let result = shorten_path(p, 15);
        // Must not exceed 15 columns regardless of byte width
        assert!(
            result.chars().count() <= 15,
            "got {} chars: {}",
            result.chars().count(),
            result
        );
    }

    #[test]
    fn shorten_path_replaces_home_prefix() {
        let home = std::env::var("HOME").unwrap_or_default();
        if home.is_empty() {
            return; // skip on systems without HOME
        }
        let p = Path::new(&home).join("myfile.conf");
        let result = shorten_path(&p, 200);
        assert!(
            result.starts_with('~'),
            "expected ~ prefix, got: {}",
            result
        );
    }

    // --- handle_input: editing mode ---

    #[test]
    fn editing_char_appends_to_input() {
        let mut view = BitcoinConfigView::new();
        view.editing = true;
        let entries = vec![entry("rpcuser", "old", true)];

        view.handle_input(key(KeyCode::Char('x')), &entries);
        assert_eq!(view.edit_input, "x");
    }

    #[test]
    fn editing_backspace_removes_last_char() {
        let mut view = BitcoinConfigView::new();
        view.editing = true;
        view.edit_input = "ab".to_string();
        let entries = vec![entry("rpcuser", "old", true)];

        view.handle_input(key(KeyCode::Backspace), &entries);
        assert_eq!(view.edit_input, "a");
    }

    #[test]
    fn editing_enter_returns_commit_action() {
        let mut view = BitcoinConfigView::new();
        view.editing = true;
        view.edit_input = "newval".to_string();
        view.selected_index = 0;
        let entries = vec![entry("rpcuser", "old", true)];

        let action = view.handle_input(key(KeyCode::Enter), &entries);
        assert!(
            matches!(action, AppAction::CommitEdit(0, ref v) if v == "newval"),
            "expected CommitEdit(0, newval)"
        );
        assert!(!view.editing);
        assert!(view.edit_input.is_empty());
    }

    #[test]
    fn editing_esc_cancels_without_committing() {
        let mut view = BitcoinConfigView::new();
        view.editing = true;
        view.edit_input = "draft".to_string();
        let entries = vec![entry("rpcuser", "old", true)];

        let action = view.handle_input(key(KeyCode::Esc), &entries);
        assert!(matches!(action, AppAction::None));
        assert!(!view.editing);
        assert!(view.edit_input.is_empty());
    }

    #[test]
    fn editing_other_key_is_noop() {
        let mut view = BitcoinConfigView::new();
        view.editing = true;
        let entries = vec![entry("rpcuser", "old", true)];

        let action = view.handle_input(key(KeyCode::F(1)), &entries);
        assert!(matches!(action, AppAction::None));
        assert!(view.editing);
    }

    // --- handle_input: browsing mode ---

    #[test]
    fn browsing_down_increments_index() {
        let mut view = BitcoinConfigView::new();
        let entries = vec![entry("a", "1", true), entry("b", "2", true)];

        view.handle_input(key(KeyCode::Down), &entries);
        assert_eq!(view.selected_index, 1);
    }

    #[test]
    fn browsing_down_clamped_at_last_entry() {
        let mut view = BitcoinConfigView::new();
        view.selected_index = 1;
        let entries = vec![entry("a", "1", true), entry("b", "2", true)];

        view.handle_input(key(KeyCode::Down), &entries);
        assert_eq!(view.selected_index, 1);
    }

    #[test]
    fn browsing_up_decrements_index() {
        let mut view = BitcoinConfigView::new();
        view.selected_index = 1;
        let entries = vec![entry("a", "1", true), entry("b", "2", true)];

        view.handle_input(key(KeyCode::Up), &entries);
        assert_eq!(view.selected_index, 0);
    }

    #[test]
    fn browsing_up_clamped_at_zero() {
        let mut view = BitcoinConfigView::new();
        view.selected_index = 0;
        let entries = vec![entry("a", "1", true)];

        view.handle_input(key(KeyCode::Up), &entries);
        assert_eq!(view.selected_index, 0);
    }

    #[test]
    fn browsing_enter_starts_editing_with_current_value() {
        let mut view = BitcoinConfigView::new();
        let entries = vec![entry("rpcuser", "alice", true)];

        view.handle_input(key(KeyCode::Enter), &entries);
        assert!(view.editing);
        assert_eq!(view.edit_input, "alice");
    }

    #[test]
    fn browsing_enter_noop_when_entries_empty() {
        let mut view = BitcoinConfigView::new();
        let entries: Vec<ConfigEntry> = vec![];

        view.handle_input(key(KeyCode::Enter), &entries);
        assert!(!view.editing);
    }

    #[test]
    fn browsing_s_returns_save_action() {
        let mut view = BitcoinConfigView::new();
        let entries = vec![entry("rpcuser", "alice", true)];

        let action = view.handle_input(key(KeyCode::Char('s')), &entries);
        assert!(matches!(action, AppAction::SaveBitcoinConfig));
    }

    #[test]
    fn browsing_esc_sets_sidebar_focused() {
        let mut view = BitcoinConfigView::new();
        view.sidebar_focused = false;
        let entries = vec![entry("rpcuser", "alice", true)];

        view.handle_input(key(KeyCode::Esc), &entries);
        assert!(view.sidebar_focused);
    }

    #[test]
    fn navigation_clears_save_message() {
        let entries = vec![entry("a", "1", true), entry("b", "2", true)];

        // Up clears it
        let mut view = BitcoinConfigView::new();
        view.selected_index = 1;
        view.save_message = Some("saved".to_string());
        view.handle_input(key(KeyCode::Up), &entries);
        assert!(view.save_message.is_none());

        // Down clears it
        let mut view = BitcoinConfigView::new();
        view.save_message = Some("saved".to_string());
        view.handle_input(key(KeyCode::Down), &entries);
        assert!(view.save_message.is_none());

        // Enter (start editing) clears it
        let mut view = BitcoinConfigView::new();
        view.save_message = Some("saved".to_string());
        view.handle_input(key(KeyCode::Enter), &entries);
        assert!(view.save_message.is_none());

        // Esc (back to sidebar) clears it
        let mut view = BitcoinConfigView::new();
        view.save_message = Some("saved".to_string());
        view.handle_input(key(KeyCode::Esc), &entries);
        assert!(view.save_message.is_none());
    }

    #[test]
    fn save_key_does_not_clear_save_message() {
        let mut view = BitcoinConfigView::new();
        view.save_message = Some("Configuration correctly saved".to_string());
        let entries = vec![entry("rpcuser", "alice", true)];

        let action = view.handle_input(key(KeyCode::Char('s')), &entries);
        assert!(matches!(action, AppAction::SaveBitcoinConfig));
        assert_eq!(
            view.save_message.as_deref(),
            Some("Configuration correctly saved"),
            "save_message must not be cleared when pressing s"
        );
    }

    #[test]
    fn commit_edit_clears_save_message() {
        let mut view = BitcoinConfigView::new();
        view.editing = true;
        view.edit_input = "newval".to_string();
        view.save_message = Some("saved".to_string());
        let entries = vec![entry("rpcuser", "alice", true)];

        view.handle_input(key(KeyCode::Enter), &entries);
        assert!(view.save_message.is_none());
    }

    #[test]
    fn unrecognised_key_preserves_save_message() {
        let mut view = BitcoinConfigView::new();
        view.save_message = Some("saved".to_string());
        let entries = vec![entry("rpcuser", "alice", true)];

        view.handle_input(key(KeyCode::F(1)), &entries);
        assert_eq!(view.save_message.as_deref(), Some("saved"));
    }
}
