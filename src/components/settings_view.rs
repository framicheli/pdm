// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::app::{App, AppAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

/// Number of settings fields.
pub const FIELD_COUNT: usize = 5;

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
    pub sidebar_focused: bool,
    pub save_error: Option<String>,
}

impl SettingsView {
    #[must_use]
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            sidebar_focused: true,
            save_error: None,
        }
    }

    /// Called only when the settings content panel is focused (`sidebar_focused` = false).
    pub fn handle_input(&mut self, key: KeyEvent) -> AppAction {
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
            KeyCode::Enter => {
                if self.selected_index < 4 {
                    AppAction::OpenExplorerForSettings(self.selected_index)
                } else {
                    AppAction::None
                }
            }
            KeyCode::Backspace => AppAction::ClearSettingsField(self.selected_index),
            KeyCode::Esc => {
                self.sidebar_focused = true;
                AppAction::None
            }
            _ => AppAction::None,
        }
    }

    pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
        let values: [Option<String>; FIELD_COUNT] = [
            app.settings
                .bitcoin_conf_path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            app.settings
                .p2pool_conf_path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            app.settings
                .ln_conf_path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            app.settings
                .shares_market_conf_path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            app.settings
                .settings_dir_override
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
        ];

        let items: Vec<ListItem> = FIELD_LABELS
            .iter()
            .zip(values.iter())
            .map(|(label, val)| {
                let (display, style) = match val {
                    Some(v) => (
                        v.clone(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
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

        let panel_style = if app.settings_view.sidebar_focused {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Settings ")
                    .border_style(panel_style),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(list, area, &mut list_state);
    }
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn new_starts_at_first_field_sidebar_focused() {
        let view = SettingsView::new();
        assert_eq!(view.selected_index, 0);
        assert!(view.sidebar_focused);
    }

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
    fn browsing_enter_opens_explorer_for_file_fields() {
        let mut view = SettingsView::new();
        for idx in 0..4 {
            view.selected_index = idx;
            let action = view.handle_input(key(KeyCode::Enter));
            assert!(
                matches!(action, AppAction::OpenExplorerForSettings(i) if i == idx),
                "expected OpenExplorerForSettings({idx})"
            );
        }
    }

    #[test]
    fn browsing_enter_on_dir_override_is_noop() {
        let mut view = SettingsView::new();
        view.selected_index = 4;
        let action = view.handle_input(key(KeyCode::Enter));
        assert!(matches!(action, AppAction::None));
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

    #[test]
    fn backspace_returns_clear_for_any_field() {
        let mut view = SettingsView::new();
        for idx in 0..FIELD_COUNT {
            view.selected_index = idx;
            let action = view.handle_input(key(KeyCode::Backspace));
            assert!(
                matches!(action, AppAction::ClearSettingsField(i) if i == idx),
                "expected ClearSettingsField({idx})"
            );
        }
    }
}
