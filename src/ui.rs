use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
};

use crate::app::{App, CurrentScreen};

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(frame.area());

    // Title and Health Status
    let health_text = match app.health_status {
        Some(true) => " [Running]",
        Some(false) => " [Stopped]",
        None => " [Unknown]",
    };
    let health_color = match app.health_status {
        Some(true) => Color::Green,
        Some(false) => Color::Red,
        None => Color::Yellow,
    };

    let title = Paragraph::new("PDM - P2Poolv2 Deployment and Management")
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(title, chunks[0]);

    let health_width = health_text.len() as u16 + 2;
    let health_rect = Rect {
        x: chunks[0].x + chunks[0].width - health_width - 1,
        y: chunks[0].y + 1,
        width: health_width,
        height: 1,
    };

    let health_p = Paragraph::new(health_text).style(
        Style::default()
            .fg(health_color)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(health_p, health_rect);

    match app.current_screen {
        CurrentScreen::Main => render_main_screen(frame, app, chunks[1]),
        CurrentScreen::FileExplorer => render_file_explorer(frame, app, chunks[1]),
        CurrentScreen::Editing => render_editing_screen(frame, app, chunks[1]),
        CurrentScreen::EditingValue => {
            render_editing_screen(frame, app, chunks[1]); // Render background
            render_editing_value_popup(frame, app, chunks[1]);
        }
    }

    if let Some(msg) = &app.notification {
        let footer = Paragraph::new(msg.as_str()).style(
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(footer, chunks[2]);
    } else {
        let footer_text = match app.current_screen {
            CurrentScreen::Main => "Press 'q' to quit, Enter to select",
            CurrentScreen::FileExplorer => "Press Esc to cancel, Enter to select",
            CurrentScreen::Editing => {
                "Esc: Back, Tab: Next Section, Enter: Edit, Space: Toggle, Ctrl+S: Save"
            }
            CurrentScreen::EditingValue => "Esc: Cancel, Enter: Save",
        };
        let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(footer, chunks[2]);
    }
}

fn render_main_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3), // Button height
            Constraint::Percentage(40),
        ])
        .split(area);

    let centered_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40), // Button width
            Constraint::Percentage(30),
        ])
        .split(layout[1]);

    let button_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Green)
        .add_modifier(Modifier::BOLD);

    let button = Paragraph::new("Select Config")
        .block(Block::default().borders(Borders::ALL))
        .style(button_style)
        .alignment(Alignment::Center);

    frame.render_widget(button, centered_layout[1]);
    app.interactive_rects.select_config_button = Some(centered_layout[1]);

    if let Some(path) = &app.config_file_path {
        let path_text = Paragraph::new(format!("Selected: {}", path)).alignment(Alignment::Center);
        frame.render_widget(path_text, layout[2]);
    }
}

fn render_file_explorer(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .file_explorer
        .files
        .iter()
        .map(|path| {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("..");
            let style = if path.is_dir() {
                Style::default().fg(Color::Blue)
            } else {
                Style::default().fg(Color::White)
            };
            let display_name = if path.is_dir() {
                format!("{}/", name)
            } else {
                name.to_string()
            };
            ListItem::new(display_name).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(format!(
            "File Explorer: {}",
            app.file_explorer.current_dir.display()
        )))
        .highlight_style(
            Style::default()
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, area, &mut app.file_explorer.list_state);
    app.interactive_rects.file_list = Some(area);
}

fn render_editing_screen(frame: &mut Frame, app: &mut App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Main content
        ])
        .split(area);

    // Tabs
    let section_names: Vec<String> = app.sections.iter().map(|s| s.name.clone()).collect();
    let tabs = Tabs::new(section_names)
        .block(Block::default().borders(Borders::ALL).title("Sections"))
        .select(app.selected_section_index)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, layout[0]);
    app.interactive_rects.tabs = Some(layout[0]);

    if app.sections.is_empty() {
        let msg = Paragraph::new("No configuration loaded or empty.").alignment(Alignment::Center);
        frame.render_widget(msg, layout[1]);
        return;
    }

    let current_section = &app.sections[app.selected_section_index];

    // List (Left) and Details (Right)
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    // List of items
    let items: Vec<ListItem> = current_section
        .items
        .iter()
        .map(|entry| {
            let status = if entry.enabled { "[x]" } else { "[ ]" };
            let line = format!("{} {} = {}", status, entry.key, entry.value);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Options"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, content_layout[0], &mut app.config_list_state);
    app.interactive_rects.config_list = Some(content_layout[0]);

    // Details panel
    let selected_entry = current_section.items.get(app.selected_item_index);
    if let Some(entry) = selected_entry {
        let description = if let Some(schema) = &entry.schema {
            schema.description.as_str()
        } else {
            "Custom configuration option."
        };

        let type_info = if let Some(schema) = &entry.schema {
            format!("{:?}", schema.value_type)
        } else {
            "Unknown".to_string()
        };

        let details = vec![
            format!("Key: {}", entry.key),
            format!("Value: {}", entry.value),
            format!("Type: {}", type_info),
            String::new(),
            "Description:".to_string(),
            description.to_string(),
        ]
        .join("\n");

        let paragraph = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, content_layout[1]);
    }
}

fn render_editing_value_popup(frame: &mut Frame, app: &App, area: Rect) {
    let area = centered_rect(60, 20, area);
    frame.render_widget(Clear, area);

    let block = Block::default().title("Edit Value").borders(Borders::ALL);
    let input = Paragraph::new(app.editing_value.as_str())
        .block(block)
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
