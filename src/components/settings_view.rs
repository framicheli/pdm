// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::{App, AppAction};
use crate::settings::config_dir;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

/// Number of editable settings fields. Used by the key handler in main.rs.
pub const FIELD_COUNT: usize = 5;

/// Labels and field count must stay in sync with `field_value` / `CommitSettingsEdit` handler.
const FIELD_LABELS: [&str; FIELD_COUNT] = [
    "Bitcoin config path",
    "P2Pool config path",
    "LN config path",
    "Shares Market config path",
    "Settings directory override",
];

#[derive(Debug, Clone)]
pub struct SettingsView {
    pub selected_index: usize,
    pub editing: bool,
    pub edit_input: String,
    pub save_message: Option<String>,
    pub sidebar_focused: bool,
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            editing: false,
            edit_input: String::new(),
            save_message: None,
            sidebar_focused: true,
        }
    }

    /// Called only when the settings content panel is focused (sidebar_focused = false).
    pub fn handle_input(&mut self, key: KeyEvent) -> AppAction {
        self.save_message = None;

        if self.editing {
            match key.code {
                KeyCode::Enter => {
                    let action = AppAction::CommitSettingsEdit(
                        self.selected_index,
                        self.edit_input.trim().to_string(),
                    );
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
                    if self.selected_index + 1 < FIELD_LABELS.len() {
                        self.selected_index += 1;
                    }
                    AppAction::None
                }
                KeyCode::Enter => AppAction::BeginSettingsEdit(self.selected_index),
                KeyCode::Char('s') => AppAction::SaveSettings,
                KeyCode::Esc => {
                    self.sidebar_focused = true;
                    AppAction::None
                }
                _ => AppAction::None,
            }
        }
    }

    pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area);

        // Collect current field values from app state
        let values: [Option<String>; 5] = [
            app.settings.bitcoin_conf_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            app.settings.p2pool_conf_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            app.settings.ln_conf_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            app.settings.shares_market_conf_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            app.settings.settings_dir_override.as_ref().map(|p| p.to_string_lossy().into_owned()),
        ];

        let items: Vec<ListItem> = FIELD_LABELS
            .iter()
            .zip(values.iter())
            .map(|(label, val)| {
                let (display, style) = match val {
                    Some(v) => (
                        v.clone(),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    ),
                    None => (
                        "(not set)".to_string(),
                        Style::default().fg(Color::DarkGray),
                    ),
                };
                ListItem::new(vec![
                    Line::from(Span::styled(*label, Style::default().fg(Color::Gray))),
                    Line::from(Span::styled(display, style)),
                ])
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(app.settings_view.selected_index));

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Settings "))
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(list, panels[0], &mut list_state);

        // Right panel: detail + edit input
        let right_block = Block::default().borders(Borders::ALL).title(" Detail ");
        let inner = right_block.inner(panels[1]);
        f.render_widget(right_block, panels[1]);

        let selected = app.settings_view.selected_index;
        let editing = app.settings_view.editing;
        let edit_input = app.settings_view.edit_input.clone();
        let save_message = app.settings_view.save_message.clone();

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // field label
                Constraint::Length(1), // spacer
                Constraint::Length(1), // "Value:" label
                Constraint::Length(3), // input box
                Constraint::Length(1), // spacer
                Constraint::Length(2), // hint / save message
                Constraint::Min(0),
            ])
            .split(inner);

        f.render_widget(
            Paragraph::new(FIELD_LABELS[selected]).style(Style::default().fg(Color::White)),
            rows[0],
        );
        f.render_widget(
            Paragraph::new("Value:").style(Style::default().fg(Color::Gray)),
            rows[2],
        );

        if editing {
            f.render_widget(
                Paragraph::new(format!("{}_", edit_input))
                    .block(Block::default().borders(Borders::ALL))
                    .style(Style::default().fg(Color::Yellow)),
                rows[3],
            );
        } else {
            let (display, style) = match &values[selected] {
                Some(v) => (
                    v.clone(),
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                ),
                None => (
                    "(not set)".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
            };
            f.render_widget(
                Paragraph::new(display)
                    .block(Block::default().borders(Borders::ALL))
                    .style(style),
                rows[3],
            );
        }

        // Hint: settings dir override takes effect on restart
        if selected == 4 {
            f.render_widget(
                Paragraph::new("Takes effect on next restart.")
                    .style(Style::default().fg(Color::DarkGray)),
                rows[5],
            );
        }

        if let Some(msg) = save_message {
            f.render_widget(
                Paragraph::new(format!("✓ {}", msg)).style(Style::default().fg(Color::Green)),
                rows[5],
            );
        }
    }
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}

// Helper: resolve the effective settings dir for display in the detail panel.
// Returns the platform default if no override is set.
pub fn effective_config_dir_display() -> String {
    config_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "(unavailable)".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    // --- SettingsView::new / default ---

    #[test]
    fn new_starts_at_first_field_not_editing() {
        let view = SettingsView::new();
        assert_eq!(view.selected_index, 0);
        assert!(!view.editing);
        assert!(view.edit_input.is_empty());
        assert!(view.save_message.is_none());
    }

    // --- browsing mode ---

    #[test]
    fn browsing_down_increments_index() {
        let mut view = SettingsView::new();
        view.handle_input(key(KeyCode::Down));
        assert_eq!(view.selected_index, 1);
    }

    #[test]
    fn browsing_down_clamped_at_last_field() {
        let mut view = SettingsView::new();
        view.selected_index = FIELD_COUNT - 1;
        view.handle_input(key(KeyCode::Down));
        assert_eq!(view.selected_index, FIELD_COUNT - 1);
    }

    #[test]
    fn browsing_up_decrements_index() {
        let mut view = SettingsView::new();
        view.selected_index = 2;
        view.handle_input(key(KeyCode::Up));
        assert_eq!(view.selected_index, 1);
    }

    #[test]
    fn browsing_up_clamped_at_zero() {
        let mut view = SettingsView::new();
        view.selected_index = 0;
        view.handle_input(key(KeyCode::Up));
        assert_eq!(view.selected_index, 0);
    }

    #[test]
    fn browsing_enter_returns_begin_settings_edit() {
        let mut view = SettingsView::new();
        view.selected_index = 2;
        let action = view.handle_input(key(KeyCode::Enter));
        assert!(matches!(action, AppAction::BeginSettingsEdit(2)));
    }

    #[test]
    fn browsing_s_returns_save_settings() {
        let mut view = SettingsView::new();
        let action = view.handle_input(key(KeyCode::Char('s')));
        assert!(matches!(action, AppAction::SaveSettings));
    }

    #[test]
    fn browsing_esc_sets_sidebar_focused_flag() {
        let mut view = SettingsView::new();
        view.sidebar_focused = false;
        let action = view.handle_input(key(KeyCode::Esc));
        assert!(matches!(action, AppAction::None));
        assert!(view.sidebar_focused);
    }

    #[test]
    fn browsing_other_key_is_noop() {
        let mut view = SettingsView::new();
        let action = view.handle_input(key(KeyCode::F(5)));
        assert!(matches!(action, AppAction::None));
    }

    // --- editing mode ---

    #[test]
    fn editing_char_appends() {
        let mut view = SettingsView::new();
        view.editing = true;
        view.handle_input(key(KeyCode::Char('/')));
        view.handle_input(key(KeyCode::Char('t')));
        view.handle_input(key(KeyCode::Char('m')));
        view.handle_input(key(KeyCode::Char('p')));
        assert_eq!(view.edit_input, "/tmp");
    }

    #[test]
    fn editing_backspace_removes_last_char() {
        let mut view = SettingsView::new();
        view.editing = true;
        view.edit_input = "abc".to_string();
        view.handle_input(key(KeyCode::Backspace));
        assert_eq!(view.edit_input, "ab");
    }

    #[test]
    fn editing_enter_returns_commit_with_trimmed_value() {
        let mut view = SettingsView::new();
        view.editing = true;
        view.selected_index = 1;
        view.edit_input = "  /tmp/p2pool.toml  ".to_string();
        let action = view.handle_input(key(KeyCode::Enter));
        assert!(
            matches!(action, AppAction::CommitSettingsEdit(1, ref v) if v == "/tmp/p2pool.toml"),
            "expected CommitSettingsEdit(1, trimmed)"
        );
        assert!(!view.editing);
        assert!(view.edit_input.is_empty());
    }

    #[test]
    fn editing_esc_cancels() {
        let mut view = SettingsView::new();
        view.editing = true;
        view.edit_input = "draft".to_string();
        let action = view.handle_input(key(KeyCode::Esc));
        assert!(matches!(action, AppAction::None));
        assert!(!view.editing);
        assert!(view.edit_input.is_empty());
    }

    #[test]
    fn editing_other_key_is_noop() {
        let mut view = SettingsView::new();
        view.editing = true;
        let action = view.handle_input(key(KeyCode::F(1)));
        assert!(matches!(action, AppAction::None));
        assert!(view.editing);
    }

    #[test]
    fn any_key_clears_save_message() {
        let mut view = SettingsView::new();
        view.save_message = Some("saved".to_string());
        view.handle_input(key(KeyCode::Up));
        assert!(view.save_message.is_none());
    }

    // --- effective_config_dir_display ---

    #[test]
    fn effective_config_dir_display_returns_non_empty() {
        let s = effective_config_dir_display();
        assert!(!s.is_empty());
    }
}
