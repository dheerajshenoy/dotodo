use std::fmt::write;
use std::fs;
use std::io::{self, BufReader};
use color_eyre::owo_colors::OwoColorize;
use ratatui::widgets::Padding;
use serde::{Deserialize, Serialize};
use std::vec;
use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};

use crate::tui;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Rect, Layout, Direction, Constraint},
    style::{Style, Stylize, Modifier, Color},
    symbols::border,
    text::{Line, Text, Span},
    widgets::{
        block::{Position, Title},
        Block, Paragraph, Widget,
        BorderType, List, ListDirection, ListItem, Borders
    },
    Frame,
};


enum Mode {
    Normal,
    Rename,
}

#[derive(Serialize, Deserialize, Debug)]
enum Priority {
    Normal,
    Important,
    VeryImportant,
}

#[derive(Serialize, Deserialize, Debug)]
struct TodoItem {
    title: String,
    date: String,
    deadline: String,
    priority: u8,
}

#[derive(Debug, Deserialize)]
struct TodoList {
    items: Vec<TodoItem>,
}

pub struct App {
    file: fs::File,
    todo_list: TodoList,
    exit: bool,
    sel_index: usize,
    mark_done_list: Vec<usize>,
    mode: Mode,
}

impl App {

    // Initializer function that takes in a `todo_file` argument
    pub fn new(todo_file_name: &str) -> Self {

        let file = fs::File::open(todo_file_name).expect("Unable to open the todo file");
        let reader = BufReader::new(&file);
        let todo_list = serde_json::from_reader(reader).expect("Unable to read the todo file");

        App {
            file,
            todo_list,
            exit: false,
            sel_index: 0,
            mark_done_list: vec![],
            mode: Mode::Normal
        }
    }

    // List all the todo items
    pub fn list_todo_items(self) {
        for i in self.todo_list.items {
            println!("{}", i.title);
        }
    }
    
    pub fn get_titles_from_todo_items(&self) -> Vec<String> {
        self.todo_list.items.iter().map(|item| item.title.clone()).collect()
    }

    // Check if todo item `item_name` already exists in the todo_file
    fn check_if_todo_item_exists(&self, item_name: &str) -> bool {
        self.todo_list.items.iter().any(| element | element.title == item_name)
    }

    // Add todo items
    pub fn add_todo_item(&mut self, title: String) {

        // If the item doesn't already exist
        if !self.check_if_todo_item_exists(&title) {
        }
    }

    // runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {

        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => self
                .handle_key_event(key_event)
                .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('j') => self.next_item(),
            KeyCode::Char('k') => self.prev_item(),
            KeyCode::Char('m') => self.mark_as_done(),
            KeyCode::Char('d') => self.delete_marks(),
            KeyCode::Char('r') => self.rename_marks(),
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn next_item(&mut self) {
        if self.todo_list.items.len() == 0 { return; }
        if self.sel_index < self.todo_list.items.len() - 1 {
            self.sel_index += 1;
        }
    }

    fn prev_item(&mut self) {
        if self.todo_list.items.len() == 0 { return; }
        if self.sel_index > 0 {
            self.sel_index -= 1;
        }
    }

    fn is_marked_done(&self, index: usize) -> bool {
        if self.todo_list.items.len() == 0 { return false; }
        self.mark_done_list.contains(&index)
    }

    fn mark_as_done(&mut self) {
        if self.todo_list.items.len() == 0 { return; }
        // If it has been marked, remove the mark, else mark it
        if self.is_marked_done(self.sel_index) {
            self.mark_done_list.retain(|&x| x != self.sel_index);
        } else {
            self.mark_done_list.push(self.sel_index)
        }
    }

    fn select_item(&mut self) {
        if self.todo_list.items.len() == 0 { return; }
        todo!("Haven't implemented item selection");
    }

    fn delete_marks(&mut self) {
        if self.todo_list.items.len() == 0 { return; }

        // If empty, delete the current item
        if self.mark_done_list.is_empty() {
            if self.sel_index == self.todo_list.items.len() - 1 {
                self.todo_list.items.remove(self.sel_index);
                if !self.todo_list.items.is_empty() {
                    self.sel_index -= 1;
                }
            } else {
                self.todo_list.items.remove(self.sel_index);
            }
        } else {
            let mut indices: Vec<usize> = self.mark_done_list.clone();
            indices.sort_unstable_by(|a, b| b.cmp(a)); // Sort in descending order

            for index in indices {
                if index < self.todo_list.items.len() {
                    self.todo_list.items.remove(index);
                } else {
                    eprintln!("Warning: Index {} out of bounds", index);
                }
            }

            // Optionally, clear mark_done_list after deletion
            self.mark_done_list.clear();
        }
    }

    fn rename_marks(&mut self) {
        if self.todo_list.items.len() == 0 { return; }

    }

    // Render the TUI frame
    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // 50% for the List
                Constraint::Percentage(50), // 50% for the right Block
            ].as_ref())
            .split(area);

        let instructions = Title::from(Line::from(vec![
            " Next Item ".into(),
            " j ".blue().bold(),
            " Prev Item ".into(),
            " k ".blue().bold(),
            " Select Item ".into(),
            " Enter ".blue().bold(),
            " Quit ".into(),
            " q ".blue().bold(),
        ]));
        

        let list_block = Block::default()
            .title(" DoToDo ")
            .padding(Padding::uniform(1))
            .borders(Borders::NONE);

        // TODO LIST ITEMS
        let todo_items = self.get_titles_from_todo_items();

        if todo_items.len() > 0 {

            let list_items : Vec<ListItem> = todo_items
                .iter()
                .enumerate()
                .map(|(i, title)| {
                    let style = if i == self.sel_index {
                        if self.is_marked_done(i) {
                            Style::default().black().on_blue().add_modifier(Modifier::ITALIC).crossed_out()
                        }
                        else {
                            Style::default().black().on_blue().add_modifier(Modifier::ITALIC)
                        }
                    } else {
                        if self.is_marked_done(i) {
                            Style::default().black().on_blue().add_modifier(Modifier::ITALIC).crossed_out()
                        } else {
                            Style::default()
                        }
                    };

                    ListItem::new(Span::from(Span::styled(format!(" {} ", title.clone()), style)))
                })
                .collect();

            let list = List::new(list_items)
                .block(list_block)
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);
            list.render(chunks[0], buf);
        } else {
            list_block.render(chunks[0], buf);
        }

        // Right side - Another Block for details or additional info
        let right_block = Block::default()
            .borders(Borders::ALL)
            .title(instructions)
            .title_position(Position::Bottom)
            .padding(Padding::uniform(1))
            .style(Style::default().fg(Color::White));

        // Example: Display the details of the selected todo item
        if self.todo_list.items.len() > 0
        {
            let selected_item = &self.todo_list.items[self.sel_index];

            let item_description = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("TITLE: ", Style::default()),
                    Span::styled(selected_item.title.clone(), Style::default())]),

                Line::from(vec![
                    Span::styled("DATE: ", Style::default()),
                    Span::styled(selected_item.date.clone(), Style::default())]),

                Line::from(vec![
                    Span::styled("PRIORITY: ", Style::default()),
                    Span::styled(selected_item.priority.to_string().clone(), Style::default())
                ]),

                Line::from(vec![
                    Span::styled("DEADLINE: ", Style::default()),
                    Span::styled(selected_item.deadline.to_string().clone(), Style::default())
                ])])
                .block(right_block);
            item_description.render(chunks[1], buf);
        } else {
            let item_description = Paragraph::new(" No Item Selected ")
                .block(right_block);
            item_description.render(chunks[1], buf);
        }

    }
}
