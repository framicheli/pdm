use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    Terminal,
    backend::Backend,
    layout::{Position, Rect},
    widgets::ListState,
};
use std::{fs, path::PathBuf, process::Command};

use crate::{
    config::{self, ConfigEntry, ConfigType},
    ui, utils,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CurrentScreen {
    #[default]
    Main,
    FileExplorer,
    Editing,
    EditingValue,
}

#[derive(Debug, Default)]
pub struct InteractiveRects {
    pub select_config_button: Option<Rect>,
    pub file_list: Option<Rect>,
    pub tabs: Option<Rect>,
    pub config_list: Option<Rect>,
}

#[derive(Debug, Clone)]
pub struct FileExplorerState {
    pub current_dir: PathBuf,
    pub files: Vec<PathBuf>,
    pub list_state: ListState,
}

impl Default for FileExplorerState {
    fn default() -> Self {
        Self::new()
    }
}

impl FileExplorerState {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut state = Self {
            current_dir,
            files: vec![],
            list_state: ListState::default(),
        };
        state.refresh_files();
        state.list_state.select(Some(0));
        state
    }

    pub fn refresh_files(&mut self) {
        self.files.clear();
        if self.current_dir.parent().is_some() {
            self.files.push(self.current_dir.join(".."));
        }

        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            for entry in entries.flatten() {
                self.files.push(entry.path());
            }
        }
        self.files.sort_by(|a, b| {
            let a_is_dir = a.is_dir();
            let b_is_dir = b.is_dir();
            if a_is_dir && !b_is_dir {
                std::cmp::Ordering::Less
            } else if !a_is_dir && b_is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.file_name().cmp(&b.file_name())
            }
        });

        if self.files.is_empty() {
            self.list_state.select(None);
        } else {
            let selected = self.list_state.selected().unwrap_or(0);
            if selected >= self.files.len() {
                self.list_state.select(Some(self.files.len() - 1));
            } else {
                self.list_state.select(Some(selected));
            }
        }
    }

    pub fn select_next(&mut self) {
        if self.files.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.files.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_previous(&mut self) {
        if self.files.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.files.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

#[derive(Debug, Clone)]
pub struct ConfigSection {
    pub name: String,
    pub items: Vec<ConfigEntry>,
}

#[derive(Debug, Default)]
pub struct App {
    pub running: bool,
    pub current_screen: CurrentScreen,
    pub config_file_path: Option<String>,
    pub file_explorer: FileExplorerState,

    pub sections: Vec<ConfigSection>,
    pub selected_section_index: usize,
    pub config_list_state: ListState,
    pub selected_item_index: usize,

    pub editing_value: String,

    pub health_status: Option<bool>,

    pub interactive_rects: InteractiveRects,

    pub notification: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            current_screen: CurrentScreen::Main,
            config_file_path: None,
            file_explorer: FileExplorerState::new(),
            sections: vec![],
            selected_section_index: 0,
            config_list_state: ListState::default(),
            selected_item_index: 0,
            editing_value: String::new(),
            health_status: None,
            interactive_rects: InteractiveRects::default(),
            notification: None,
        }
    }

    pub fn load_config(&mut self, path: &str) -> Result<()> {
        let entries = config::parse_config(std::path::Path::new(path))?;

        let mut sections_map: std::collections::HashMap<String, Vec<ConfigEntry>> =
            std::collections::HashMap::new();

        for entry in entries {
            let section_name = if let Some(schema) = &entry.schema {
                schema.section.clone()
            } else {
                "Custom".to_string()
            };
            sections_map.entry(section_name).or_default().push(entry);
        }

        self.sections = sections_map
            .into_iter()
            .map(|(name, items)| ConfigSection { name, items })
            .collect();

        self.sections.sort_by(|a, b| a.name.cmp(&b.name));

        self.selected_section_index = 0;
        self.selected_item_index = 0;
        self.config_list_state.select(Some(0));

        self.check_health();

        Ok(())
    }

    pub fn check_health(&mut self) {
        self.health_status = None;
        let pid_val = self.find_config_value("pid");

        if let Some(pid_path_str) = pid_val {
            let mut pid_path = PathBuf::from(&pid_path_str);
            if !pid_path.is_absolute() {
                if let Some(datadir) = self.find_config_value("datadir") {
                    pid_path = PathBuf::from(datadir).join(pid_path);
                }
            }

            if pid_path.exists() {
                if let Ok(content) = fs::read_to_string(&pid_path) {
                    if let Ok(pid) = content.trim().parse::<i32>() {
                        let output = Command::new("ps").arg("-p").arg(pid.to_string()).output();

                        if let Ok(out) = output {
                            if out.status.success() {
                                self.health_status = Some(true);
                            } else {
                                self.health_status = Some(false);
                            }
                        }
                    }
                }
            } else {
                self.health_status = Some(false);
            }
        }
    }

    fn find_config_value(&self, key: &str) -> Option<String> {
        for section in &self.sections {
            for item in &section.items {
                if item.key == key && item.enabled {
                    return Some(item.value.clone());
                }
            }
        }
        None
    }

    pub fn save_config(&mut self) -> Result<()> {
        if let Some(path) = &self.config_file_path {
            let mut content = String::new();
            for section in &self.sections {
                if section.items.iter().any(|i| i.enabled) {
                    content.push_str(&format!("# Section: {}\n", section.name));
                    for entry in &section.items {
                        if entry.enabled {
                            content.push_str(&format!("{}={}\n", entry.key, entry.value));
                        }
                    }
                    content.push('\n');
                }
            }
            std::fs::write(path, content)?;
            self.notification = Some("Configuration saved successfully.".to_string());
        }
        Ok(())
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        while self.running {
            terminal.draw(|f| ui::ui(f, self))?;

            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key_event(key),
                Event::Mouse(mouse) => self.handle_mouse_event(mouse),
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_mouse_event(&mut self, mouse: event::MouseEvent) {
        if mouse.kind == event::MouseEventKind::Down(event::MouseButton::Left) {
            match self.current_screen {
                CurrentScreen::Main => {
                    if let Some(rect) = self.interactive_rects.select_config_button {
                        if rect.contains(Position {
                            x: mouse.column,
                            y: mouse.row,
                        }) {
                            self.current_screen = CurrentScreen::FileExplorer;
                            self.file_explorer.refresh_files();
                        }
                    }
                }
                CurrentScreen::FileExplorer => {
                    if let Some(rect) = self.interactive_rects.file_list {
                        if rect.contains(Position {
                            x: mouse.column,
                            y: mouse.row,
                        }) {
                            // Account for borders (1 line top/bottom)
                            if mouse.row > rect.y && mouse.row < rect.y + rect.height - 1 {
                                let offset = self.file_explorer.list_state.offset();
                                let row_index =
                                    (mouse.row as usize).saturating_sub(rect.y as usize + 1);
                                let index = row_index + offset;

                                if index < self.file_explorer.files.len() {
                                    self.file_explorer.list_state.select(Some(index));
                                }
                            }
                        }
                    }
                }
                CurrentScreen::Editing => {
                    if let Some(rect) = self.interactive_rects.tabs {
                        if rect.contains(Position {
                            x: mouse.column,
                            y: mouse.row,
                        }) {
                            if mouse.row > rect.y && mouse.row < rect.y + rect.height - 1 {
                                let mut x = rect.x + 1;
                                for (i, section) in self.sections.iter().enumerate() {
                                    let width = section.name.len() as u16 + 3;
                                    if mouse.column >= x && mouse.column < x + width {
                                        self.selected_section_index = i;
                                        self.selected_item_index = 0;
                                        self.config_list_state.select(Some(0));
                                        break;
                                    }
                                    x += width + 1;
                                }
                            }
                        }
                    }

                    if let Some(rect) = self.interactive_rects.config_list {
                        if rect.contains(Position {
                            x: mouse.column,
                            y: mouse.row,
                        }) {
                            if mouse.row > rect.y && mouse.row < rect.y + rect.height - 1 {
                                let offset = self.config_list_state.offset();
                                let row_index =
                                    (mouse.row as usize).saturating_sub(rect.y as usize + 1);
                                let index = row_index + offset;

                                if let Some(section) =
                                    self.sections.get(self.selected_section_index)
                                {
                                    if index < section.items.len() {
                                        self.selected_item_index = index;
                                        self.config_list_state.select(Some(index));
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        if self.current_screen == CurrentScreen::Editing
            && key.modifiers.contains(KeyModifiers::CONTROL)
            && key.code == KeyCode::Char('s')
        {
        } else {
            self.notification = None;
        }

        match self.current_screen {
            CurrentScreen::Main => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.running = false,
                KeyCode::Enter => {
                    self.current_screen = CurrentScreen::FileExplorer;
                    self.file_explorer.refresh_files();
                }
                _ => {}
            },
            CurrentScreen::FileExplorer => match key.code {
                KeyCode::Esc => {
                    self.current_screen = CurrentScreen::Main;
                }
                KeyCode::Up | KeyCode::Char('k') => self.file_explorer.select_previous(),
                KeyCode::Down | KeyCode::Char('j') => self.file_explorer.select_next(),
                KeyCode::Enter => {
                    if let Some(selected_index) = self.file_explorer.list_state.selected() {
                        if let Some(selected) = self.file_explorer.files.get(selected_index) {
                            if selected.file_name().and_then(|n| n.to_str()) == Some("..")
                                || selected.ends_with("..")
                            {
                                if let Some(parent) = self.file_explorer.current_dir.parent() {
                                    self.file_explorer.current_dir = parent.to_path_buf();
                                    self.file_explorer.list_state.select(Some(0));
                                    self.file_explorer.refresh_files();
                                }
                            } else if selected.is_dir() {
                                self.file_explorer.current_dir = selected.clone();
                                self.file_explorer.list_state.select(Some(0));
                                self.file_explorer.refresh_files();
                            } else {
                                let path_str = selected.to_string_lossy().to_string();
                                self.config_file_path = Some(path_str.clone());
                                if let Err(_) = self.load_config(&path_str) {
                                } else {
                                    self.current_screen = CurrentScreen::Editing;
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            CurrentScreen::Editing => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
                    let _ = self.save_config();
                    return;
                }
                match key.code {
                    KeyCode::Esc => {
                        self.current_screen = CurrentScreen::Main;
                    }
                    KeyCode::Right | KeyCode::Tab => {
                        if !self.sections.is_empty() {
                            if self.selected_section_index < self.sections.len() - 1 {
                                self.selected_section_index += 1;
                            } else {
                                self.selected_section_index = 0;
                            }
                            self.selected_item_index = 0;
                            self.config_list_state.select(Some(0));
                        }
                    }
                    KeyCode::Left | KeyCode::BackTab => {
                        if !self.sections.is_empty() {
                            if self.selected_section_index > 0 {
                                self.selected_section_index -= 1;
                            } else {
                                self.selected_section_index = self.sections.len() - 1;
                            }
                            self.selected_item_index = 0;
                            self.config_list_state.select(Some(0));
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(section) = self.sections.get(self.selected_section_index) {
                            if !section.items.is_empty() {
                                if self.selected_item_index < section.items.len() - 1 {
                                    self.selected_item_index += 1;
                                    self.config_list_state
                                        .select(Some(self.selected_item_index));
                                }
                            }
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if self.selected_item_index > 0 {
                            self.selected_item_index -= 1;
                            self.config_list_state
                                .select(Some(self.selected_item_index));
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(section) = self.sections.get_mut(self.selected_section_index) {
                            if let Some(entry) = section.items.get_mut(self.selected_item_index) {
                                let is_bool = if let Some(schema) = &entry.schema {
                                    schema.value_type == ConfigType::Boolean
                                } else {
                                    false
                                };

                                if is_bool {
                                    entry.value = if entry.value == "1" {
                                        "0".into()
                                    } else {
                                        "1".into()
                                    };
                                    entry.enabled = true;
                                } else {
                                    self.editing_value = entry.value.clone();
                                    self.current_screen = CurrentScreen::EditingValue;
                                }
                            }
                        }
                    }
                    KeyCode::Char(' ') => {
                        if let Some(section) = self.sections.get_mut(self.selected_section_index) {
                            if let Some(entry) = section.items.get_mut(self.selected_item_index) {
                                entry.enabled = !entry.enabled;
                            }
                        }
                    }
                    _ => {}
                }
            }
            CurrentScreen::EditingValue => match key.code {
                KeyCode::Esc => {
                    self.current_screen = CurrentScreen::Editing;
                }
                KeyCode::Enter => {
                    if let Some(section) = self.sections.get_mut(self.selected_section_index) {
                        if let Some(entry) = section.items.get_mut(self.selected_item_index) {
                            entry.value = self.editing_value.clone();
                            entry.enabled = true;
                        }
                    }
                    self.current_screen = CurrentScreen::Editing;
                }
                KeyCode::Backspace => {
                    self.editing_value.pop();
                }
                KeyCode::Char(c) => {
                    self.editing_value.push(c);
                }
                _ => {}
            },
        }
    }
}
