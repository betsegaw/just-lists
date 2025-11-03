use color_eyre::{Result, eyre::Ok};
use core::panic;
use crossterm::event::{self, Event, KeyCode};
use just_lists_core::{get_sample_list, list::List};
use ratatui::prelude::*;
use ratatui::widgets::ListState;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;
use std::collections::HashSet;

use ratatui::{
    DefaultTerminal, Frame,
    layout::Flex,
    widgets::Paragraph,
    widgets::{Block, Borders, Clear, List as WidgetList, ListItem},
};

use clap::Parser;

#[derive(Parser)]
struct Inputs {
    file: Option<PathBuf>,
}

#[derive(Clone)]
struct ListEntry {
    id_path: Vec<String>,
    expanded: bool,
}

enum UIState {
    ListView,
    EditView,
}

#[derive(Debug)]
enum Message {
    Up,
    Down,
    Esc,
    Edit,
    Enter,
    Space,
    Left,
    Right,
    Backspace,
    New,
    InsertChild,
    Delete,
    FocusOnCurrentItem,
    FocusOnParentItem,
    Copy,
    Cut,
    Paste,
    Text(char),
}

enum ClipboardAction {
    Cut(Option<String>),
    Copy,
}

struct Clipboard {
    action_type: ClipboardAction,
    list_item_id: String,
}

pub struct App {
    list: List,
    state: UIState,
    selected_list_index: usize,
    cursor_index: i32,
    display: Vec<ListEntry>,
    edit_text: String,
    file_path: Option<PathBuf>,
    display_parent_item: Option<Vec<String>>,
    clipboard: Option<Clipboard>,
    debug: bool,
    expanded_items: HashSet<Vec<String>>,
}

impl App {
    pub fn new() -> App {
        let inputs = Inputs::parse();
        let mut list: List;
        if inputs.file.is_some() {
            list = List::from_string(App::open_or_create_file(&inputs.file.clone().unwrap()));
        } else {
            list = get_sample_list();
        }

        let list_is_empty = list.get_top_level_list_items().len() == 0;

        if list_is_empty {
            list.add_list_item(just_lists_core::list_item::ListItem::new("".to_string()));
        }

        let app = App {
            list: list,
            state: if !list_is_empty {
                UIState::ListView
            } else {
                UIState::EditView
            },
            selected_list_index: 0,
            display: Vec::new(),
            cursor_index: 0,
            edit_text: "".to_string(),
            file_path: inputs.file,
            display_parent_item: None,
            clipboard: None,
            debug: false,
            expanded_items: HashSet::new(),
        };

        app
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.update_display(None);

        loop {
            terminal.draw(|f| self.view(f))?;
            let current_msg = self.handle_event()?;

            match self.state {
                UIState::ListView => match current_msg {
                    Some(Message::Esc) => return Ok(()),
                    Some(Message::Down) => self.handle_scroll(Message::Down),
                    Some(Message::Up) => self.handle_scroll(Message::Up),
                    Some(Message::Enter) => self.handle_expand(),
                    Some(Message::FocusOnCurrentItem) => self.focus_on_current(),
                    Some(Message::FocusOnParentItem) => self.focus_on_parent(),
                    Some(Message::Copy) => self.copy(),
                    Some(Message::Cut) => self.cut(),
                    Some(Message::Paste) => self.paste(),
                    Some(Message::Delete) => self.delete_selected_item(),
                    Some(Message::New) => self.add_new_list_item(),
                    Some(Message::InsertChild) => self.insert_child_item(),
                    Some(Message::Space) => self.toggle_item_completion(),
                    Some(Message::Edit) => self.toggle_edit_mode(),
                    Some(Message::Text(c)) => self.handle_text_input(c),
                    None => (),
                    _ => (),
                },
                UIState::EditView => match current_msg {
                    Some(Message::Esc) => self.state = UIState::ListView,
                    Some(Message::Enter) => self.save_edited_text(),
                    Some(Message::Left) => self.handle_cursor_left(),
                    Some(Message::Right) => self.handle_cursor_right(),
                    Some(Message::Backspace) => self.handle_backspace(),
                    Some(Message::Text(c)) => self.handle_text_input(c),
                    _ => (),
                },
            }
        }
    }

    const SELECTED_STYLE: Style = Style::new()
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD);

    fn view(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Percentage(100)])
            .split(frame.area());

        let current_path = match self.file_path.clone() {
            Some(path) => path.to_str().unwrap().to_string(),
            None => "Sample".to_string(),
        };

        let title_block = Block::new().borders(Borders::ALL);

        let mut title_paragraph: Paragraph;

        match &self.display_parent_item {
            None => {
                title_paragraph = Paragraph::new(current_path);
            }
            Some(parent_item) => {
                let mut path = current_path;
                for id in parent_item {
                    let item = self.list.get_list_item(&id).unwrap();
                    path.push('/');
                    path.push_str(item.value.clone().as_str());
                }

                title_paragraph = Paragraph::new(path);
            }
        };

        title_paragraph = title_paragraph.block(title_block);

        frame.render_widget(title_paragraph, layout[0]);

        let block = Block::new().borders(Borders::ALL);

        let items: Vec<ListItem> = self
            .display
            .iter()
            .enumerate()
            .map(|(i, todo_item)| {
                let text: String;
                let list_item = self
                    .list
                    .get_list_item(todo_item.id_path.last().unwrap())
                    .unwrap();
                let (color, check_box_state) = if list_item.completed {
                    (Color::Green, "✔ ")
                } else {
                    (Color::Gray, "☐ ")
                };

                let children_count = self.list.get_children(list_item).len();
                let children_count_text = if children_count > 0 {
                    format!("({}) ", children_count)
                } else {
                    "".to_string()
                };

                let debug_text = if self.debug {
                    format!("({})", todo_item.id_path.join("/"))
                } else {
                    "".to_string()
                };

                let parent_path_length =
                    if let Some(display_path) = self.display_parent_item.clone() {
                        display_path.len()
                    } else {
                        0
                    };

                text = format!(
                    "  {}{}{}{}{}",
                    "  ".repeat(todo_item.id_path.len() - parent_path_length - 1),
                    check_box_state,
                    children_count_text,
                    list_item.value.clone(),
                    debug_text
                );

                let mut item = ListItem::from(text).style(color);

                if i == self.selected_list_index {
                    item = item.style(Self::SELECTED_STYLE);
                }

                return item;
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = WidgetList::new(items).block(block).highlight_symbol(">");
        let mut list_state = ListState::default();

        list_state.select(Some(self.selected_list_index));

        frame.render_stateful_widget(list, layout[1], &mut list_state);

        match self.state {
            UIState::EditView => {
                let block = Block::new().borders(Borders::ALL);
                let edit_content = Paragraph::new(self.edit_text.clone()).block(block);
                let area = Self::popup_area(frame.area(), 60, 20);
                frame.render_widget(Clear, area);
                frame.render_widget(edit_content, area);

                frame.set_cursor_position(Position::new(
                    area.x + u16::try_from(self.cursor_index).unwrap() + 1,
                    area.y + 1,
                ));
            }
            _ => (),
        }
    }

    fn handle_event(&self) -> color_eyre::Result<Option<Message>> {
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    return Ok(self.handle_key(key));
                }
            }
        }
        Ok(None)
    }

    fn handle_key(&self, key: event::KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Esc => Some(Message::Esc),
            KeyCode::Enter => Some(Message::Enter),
            KeyCode::Up => Some(Message::Up),
            KeyCode::Down => Some(Message::Down),
            KeyCode::Left => Some(Message::Left),
            KeyCode::Right => Some(Message::Right),
            KeyCode::Backspace => Some(Message::Backspace),
            KeyCode::Char(c) => match self.state {
                UIState::ListView => match key.code {
                    KeyCode::Char(' ') => Some(Message::Space),
                    KeyCode::Char('e') => Some(Message::Edit),
                    KeyCode::Char('n') => Some(Message::New),
                    KeyCode::Char('i') => Some(Message::InsertChild),
                    KeyCode::Char('d') => Some(Message::Delete),
                    KeyCode::Char('j') => Some(Message::FocusOnCurrentItem),
                    KeyCode::Char('u') => Some(Message::FocusOnParentItem),
                    KeyCode::Char('c') => Some(Message::Copy),
                    KeyCode::Char('x') => Some(Message::Cut),
                    KeyCode::Char('v') => Some(Message::Paste),
                    _ => None,
                },
                UIState::EditView => {
                    return Some(Message::Text(c));
                }
            },

            _ => None,
        }
    }

    fn handle_scroll(&mut self, message: Message) {
        if self.display.len() == 0 {
            return;
        }

        match message {
            Message::Down => {
                if self.selected_list_index == self.display.len() - 1 {
                    self.selected_list_index = 0;
                } else {
                    self.selected_list_index += 1;
                }
            }
            Message::Up => {
                if self.selected_list_index == 0 {
                    self.selected_list_index = self.display.len() - 1;
                } else {
                    self.selected_list_index -= 1;
                }
            }
            _ => panic!("Scroll can't handle this message. Message:{:?}", message),
        }
    }

    fn handle_expand(&mut self) {
        if self.display.len() == 0 {
            return;
        }

        let current_item = self.display.get_mut(self.selected_list_index).unwrap();

        if current_item.expanded == false {
            self.expanded_items.insert(current_item.id_path.clone());
            let list_item_children = self.list.get_children(
                self.list
                    .get_list_item(current_item.id_path.last().unwrap())
                    .unwrap(),
            );

            current_item.expanded = true;

            let current_item_path = current_item.id_path.clone();

            for child in list_item_children {
                let mut child_path = current_item_path.clone();
                child_path.push(child.id.clone());
                self.display.insert(
                    self.selected_list_index + 1,
                    ListEntry {
                        id_path: child_path,
                        expanded: false,
                    },
                );
            }
        } else {
            self.expanded_items.retain(|p| *p != current_item.id_path);
            current_item.expanded = false;
            let current_path_length = current_item.id_path.len();

            let mut next_cursor_index = self.selected_list_index + 1;

            while next_cursor_index != self.display.len()
                && current_path_length < self.display.get(next_cursor_index).unwrap().id_path.len()
            {
                next_cursor_index += 1;
            }

            for i in (self.selected_list_index + 1..next_cursor_index).rev() {
                self.display.remove(i);
            }
        }
    }

    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    fn get_parent_from_path(path: &Vec<String>) -> Option<&str> {
        if path.len() <= 1 {
            None
        } else {
            Some(path.get(path.len() - 2).unwrap())
        }
    }

    fn open_or_create_file(path: &std::path::PathBuf) -> String {
        let mut file: fs::File;

        if fs::exists(path).is_ok_and(|f| f) {
            file = fs::File::open(path).unwrap();
        } else {
            file = fs::File::create(path).unwrap();
        }

        let mut content: String = String::new();
        let _ = file.read_to_string(&mut content);

        content
    }

    fn save_list(&self) {
        if self.file_path.is_none() {
            return;
        }

        _ = fs::write(self.file_path.clone().unwrap(), self.list.into_string());
    }

    fn delete_selected_item(&mut self) {
        if self.display.len() == 0 {
            return;
        }

        let item_to_delete_id_path = self
            .display
            .get(self.selected_list_index)
            .unwrap()
            .id_path
            .clone();
        let item_to_delete_id = item_to_delete_id_path.last().unwrap();

        let selected_item = self.get_current_display_item();

        if let Some(item) = selected_item {
            if item.expanded {
                self.handle_expand();
            }
        }

        self.display.remove(self.selected_list_index);

        let parent = App::get_parent_from_path(&item_to_delete_id_path).map(|s| s.to_string());

        _ = self
            .list
            .remove_child_list_item(item_to_delete_id, parent.as_ref());

        if self.display.len() > 0 {
            self.selected_list_index = self.selected_list_index.clamp(0, self.display.len() - 1);
        }

        self.update_display(None);
        self.save_list();
    }

    fn add_new_list_item(&mut self) {
        let item = just_lists_core::list_item::ListItem::new("".to_string());
        let mut current_item_path: Vec<String>;

        if let Some(selected_list_entry) = self.get_current_display_item() {
            if selected_list_entry.expanded {
                self.handle_expand();
            }
        }

        if self.display.len() == 0 {
            current_item_path = vec![item.id.clone()];
            self.list.add_list_item(item.clone());
            self.display.insert(
                0,
                ListEntry {
                    id_path: current_item_path.clone(),
                    expanded: false,
                },
            )
        } else {
            current_item_path = self
                .display
                .get(self.selected_list_index)
                .unwrap()
                .id_path
                .clone();

            current_item_path.remove(current_item_path.len() - 1);
            current_item_path.push(item.id.clone());

            if current_item_path.len() == 1 {
                self.list.add_list_item(item);
            } else {
                _ = self
                    .list
                    .add_child_list_item(item, &current_item_path[current_item_path.len() - 2]);
            }

            self.display.insert(
                self.selected_list_index + 1,
                ListEntry {
                    id_path: current_item_path.clone(),
                    expanded: false,
                },
            )
        }

        self.update_display(Some(current_item_path));
        self.toggle_edit_mode();
        self.save_list();
    }

    fn insert_child_item(&mut self) {
        if self.display.len() == 0 {
            return;
        }

        let item = just_lists_core::list_item::ListItem::new("".to_string());

        let parent_path = self.display
                .get(self.selected_list_index)
                .unwrap()
                .id_path
                .clone();
        
        let mut child_path = parent_path.clone();
        child_path.push(item.id.clone());

        _ = self.list.add_child_list_item(
            item,
            parent_path
                .last()
                .unwrap(),
        );

        let current_item = self.display.get_mut(self.selected_list_index).unwrap();

        if !current_item.expanded {
            self.handle_expand();
        } else {
            self.handle_expand();
            self.handle_expand();
        }

        self.update_display(Some(child_path));
        self.toggle_edit_mode();

        self.save_list();
    }

    fn toggle_item_completion(&mut self) {
        if self.display.len() == 0 {
            return;
        }

        let list_entry = self.display.get(self.selected_list_index).unwrap();
        let list_item = self
            .list
            .get_mut_list_item(list_entry.id_path.last().unwrap())
            .unwrap();

        list_item.completed = !list_item.completed;

        self.save_list();
    }

    fn toggle_edit_mode(&mut self) {
        match self.state {
            UIState::ListView => {
                if self.display.len() == 0 {
                    return;
                }

                let list_entry = self.display.get(self.selected_list_index).unwrap();
                let list_item = self
                    .list
                    .get_list_item(list_entry.id_path.last().unwrap())
                    .unwrap();
                self.edit_text = list_item.value.clone();
                self.cursor_index = self.edit_text.len() as i32;
                self.state = UIState::EditView
            }
            UIState::EditView => self.state = UIState::ListView,
        }
    }

    fn handle_backspace(&mut self) {
        if self.edit_text.len() > 0 {
            self.edit_text.remove((self.cursor_index - 1) as usize);
        }
        self.cursor_index = (self.cursor_index - 1).clamp(0, self.edit_text.len() as i32);
    }

    fn handle_cursor_left(&mut self) {
        self.cursor_index = (self.cursor_index - 1).clamp(0, self.edit_text.len() as i32)
    }

    fn handle_cursor_right(&mut self) {
        self.cursor_index = (self.cursor_index + 1).clamp(0, self.edit_text.len() as i32)
    }

    fn handle_text_input(&mut self, c: char) {
        self.edit_text.insert(self.cursor_index as usize, c);
        self.cursor_index += 1;
    }

    fn save_edited_text(&mut self) {
        let list_item = self
            .list
            .get_mut_list_item(
                self.display
                    .get(self.selected_list_index)
                    .unwrap()
                    .id_path
                    .last()
                    .unwrap(),
            )
            .unwrap();
        list_item.value = self.edit_text.clone();
        self.state = UIState::ListView;

        self.save_list();
    }

    fn focus_on_current(&mut self) {
        let current_display_item = self.get_current_display_item();

        match current_display_item {
            None => (),
            Some(list_entry) => {
                self.display_parent_item = Some(list_entry.id_path.clone());
            }
        }

        self.update_display(None);
    }

    fn focus_on_parent(&mut self) {
        let mut new_path: Vec<String>;

        match &self.display_parent_item {
            None => {
                return;
            }
            Some(current_parent_path) => {
                new_path = current_parent_path.clone();
                _ = new_path.remove(new_path.len() - 1);
            }
        }

        if new_path.len() > 0 {
            self.display_parent_item = Some(new_path);
        } else {
            self.display_parent_item = None;
        }

        self.update_display(None);
    }

    fn get_current_display_item(&self) -> Option<&ListEntry> {
        self.display.get(self.selected_list_index)
    }

    fn copy(&mut self) {
        if let Some(item_id) = self.get_current_display_item() {
            self.clipboard = Some(Clipboard {
                action_type: ClipboardAction::Copy,
                list_item_id: item_id.id_path.last().unwrap().clone(),
            })
        }
    }

    fn cut(&mut self) {
        if let Some(item_id) = self.get_current_display_item() {
            let parent_id = Self::get_parent_from_path(&item_id.id_path).map(|p| p.to_owned());
            self.clipboard = Some(Clipboard {
                action_type: ClipboardAction::Cut(parent_id),
                list_item_id: item_id.id_path.last().unwrap().clone(),
            })
        }
    }

    fn paste(&mut self) {
        if let Some(clipboard) = &self.clipboard {
            let current_selected_item = self.get_current_display_item().cloned();
            if let Some(selected_item) = current_selected_item {
                _ = self.list.add_existing_child_list_item(
                    &clipboard.list_item_id,
                    selected_item.id_path.last().unwrap(),
                );
            }

            if let ClipboardAction::Cut(previous_parent_id) = &clipboard.action_type {
                _ = self
                    .list
                    .remove_child_list_item(&clipboard.list_item_id, previous_parent_id.as_ref());
            }
        }

        self.update_display(None);
    }

    fn update_display(&mut self, custom_selected_item: Option<Vec<String>>) {
        let items_to_display: Vec<&just_lists_core::list_item::ListItem>;

        match self.display_parent_item.clone() {
            None => {
                items_to_display = self.list.get_top_level_list_items();
            }
            Some(path) => {
                items_to_display = self
                    .list
                    .get_children(self.list.get_list_item(path.last().unwrap()).unwrap());
            }
        };

        self.display.clear();

        for c in items_to_display {
            let mut id_path = if self.display_parent_item.is_none() {
                vec![]
            } else {
                self.display_parent_item.clone().unwrap()
            };

            id_path.push(c.id.clone());

            self.display.push(ListEntry {
                id_path: id_path,
                expanded: false,
            })
        }

        if self.display.len() > 0 {
            self.selected_list_index = self.selected_list_index.clamp(0, self.display.len() - 1);
        }

        let mut display_index = 0;
        
        let old_selected_entry = self.display.get(self.selected_list_index.clone()).cloned();

        while let Some(display_item) = self.display.get(display_index) {
            if self.expanded_items.contains(&display_item.id_path) {
                self.selected_list_index = display_index;
                self.handle_expand();
            }

            display_index += 1;
        }

        match custom_selected_item {
            None => {
                    if self.display.len() > 0 {
                        if let Some(old_selected_entry) = old_selected_entry {
                            if let Some(index) = self.display.iter().position(|e| e.id_path == old_selected_entry.id_path) {
                                self.selected_list_index = index;
                            }
                        }
                    }
            },
            Some(select_id_path) => {
                display_index = 0;

                while let Some(display_item) = self.display.get(display_index) {
                    if display_item.id_path == select_id_path {
                        self.selected_list_index = display_index;
                        break;
                    }

                    display_index += 1;
                }
            }
        }
    }
}
