use std::{
    io::prelude::*,
    fs::File,
    str::Split,
};
use ratatui::{
    prelude::*,
    widgets::*,
    layout::{Offset},
};
use crossterm::event::{KeyCode};
use serde::{Deserialize, Serialize};
use chrono::{Datelike, Weekday, NaiveDate};
use crate::{
    theme::{THEME},
    popup::{PopupStatus, TaskEditorPopup, TaskSource},
    widgets::{Calendar},
};

#[derive(Default, Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Duration {
    pub days: u16,
    pub hours: u8,
    pub minutes: u8,
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.days == 0 && self.hours == 0 && self.minutes == 0 {
            write!(f, "   --   ")
        } else {
            write!(f, "{:02}:{:02}:{:02}", self.days, self.hours, self.minutes)
        }
    }
}

fn disp_md(date: NaiveDate) -> String {
    format!("{} {:02}",
        match date.month() {
            1  => "Jan",
            2  => "Feb",
            3  => "Mar",
            4  => "Apr",
            5  => "May",
            6  => "Jun",
            7  => "Jul",
            8  => "Aug",
            9  => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _  => "ERR",
        },
        date.day(),)
}

pub enum TaskCommandError {
    UnknownCommand,
    InvalidFile,
}

#[derive(Default, Clone, Deserialize, Serialize)]
pub enum TaskStatus {
    #[default]
    NotStarted,
    InProgress,
    Finished,
    Deleted,
}

impl TaskStatus {
    pub fn get_symbol(&self) -> char {
        match self {
            TaskStatus::NotStarted => ' ',
            TaskStatus::InProgress => '-',
            TaskStatus::Finished => 'x',
            TaskStatus::Deleted => 'D',
        }
    }

    pub fn cycle_next(&mut self) {
        *self = match *self {
            TaskStatus::NotStarted => TaskStatus::InProgress,
            TaskStatus::InProgress => TaskStatus::Finished,
            TaskStatus::Finished => TaskStatus::NotStarted,
            TaskStatus::Deleted => TaskStatus::Deleted,
        }
    }
}

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct Task {
    pub name: String,
    pub status: TaskStatus,
    pub duration: Duration,
    pub date: NaiveDate,
    pub sub_tasks: Vec<Task>,
}

#[derive(Deserialize, Serialize)]
pub struct TaskList {
    pub name: String,
    pub selected: usize,
    pub tasks: Vec<Task>,
}

impl TaskList {
    fn next_task(&mut self) {
        if self.tasks.is_empty() { return; }
        self.selected = (self.selected + 1) % self.tasks.len();
    }

    fn previous_task(&mut self) {
        if self.tasks.is_empty() { return; }
        self.selected = (self.selected + self.tasks.len() - 1) % self.tasks.len();
    }
}

pub struct TaskListTab {
    pub controls: [(&'static str, &'static str); 5],
    pub selected: usize,

    pub new_task_window: TaskEditorPopup,
}

impl TaskListTab {
    pub fn handle_input(&mut self, task_lists: &mut Vec<TaskList>, key: KeyCode) -> bool {
        let selected_list = &mut task_lists[self.selected];
        let mut input_captured = true;

        if PopupStatus::InUse == self.new_task_window.status {
            input_captured = self.new_task_window.handle_input(key);
            // recheck status so new task can be added on the same frame
            if PopupStatus::Confirmed == self.new_task_window.status {
                match self.new_task_window.task_source {
                    TaskSource::New => selected_list.tasks.push(self.new_task_window.take_task()),
                    TaskSource::Existing => selected_list.tasks[selected_list.selected] = self.new_task_window.take_task(),
                }
                self.new_task_window.status = PopupStatus::Closed;
            }
        } else {
            match key {
                KeyCode::Char('h') => self.previous_tab(task_lists),
                KeyCode::Char('l') => self.next_tab(task_lists),
                KeyCode::Char('k') => selected_list.previous_task(),
                KeyCode::Char('j') => selected_list.next_task(),
                KeyCode::Char('a') => self.new_task(),
                KeyCode::Char('e') => self.edit_task(task_lists),
                KeyCode::Char('m') => self.mark_task(task_lists),
                KeyCode::Char('d') => self.delete_task(task_lists),
                //KeyCode::Char('s') => self.save_data(task_lists),
                //KeyCode::Char('S') => self.load_data(task_lists),
                _ => input_captured = false,
            }
        }

        input_captured
    }

    pub fn process_command(&mut self, mut command: Split<char>, task_lists: &mut Vec<TaskList>) -> Result<(), TaskCommandError> {
        match command.next() {
            Some("new") => {
                self.new_task();
                Ok(())
            }
            Some("save") => match command.next() {
                None => {
                    self.save_data("list.json", task_lists);
                    Ok(())
                }
                Some(filename) => {
                    self.save_data(filename, task_lists);
                    Ok(())
                }
            }
            Some("load") => match command.next() {
                None => self.load_data("list.json", task_lists),
                Some(filename) => self.load_data(filename, task_lists),
            }
            None | Some(_) => Err(TaskCommandError::UnknownCommand),
        }
    }

    fn load_data(&mut self, filename: &str, task_lists: &mut Vec<TaskList>) -> Result<(), TaskCommandError> {
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(_) => return Err(TaskCommandError::InvalidFile)
        };
        let mut data = vec![];
        file.read_to_end(&mut data).unwrap();
        let temp: Vec<TaskList> = serde_json::from_slice(&data).unwrap();
        *task_lists = temp;
        Ok(())
    }

    fn save_data(&mut self, filename: &str, task_lists: &mut Vec<TaskList>) {
        let mut file = File::create(filename).unwrap();
        let out = serde_json::to_vec(&task_lists).unwrap();
        file.write_all(&out).unwrap();
    }

    fn mark_task(&mut self, task_lists: &mut Vec<TaskList>) {
        if task_lists.is_empty() || task_lists[self.selected].tasks.is_empty() { return; }

        let list = &mut task_lists[self.selected];
        list.tasks[list.selected].status.cycle_next();
    }

    fn delete_task(&mut self, task_lists: &mut Vec<TaskList>) {
        if task_lists.is_empty() || task_lists[self.selected].tasks.is_empty() { return; }

        let selected_list = &mut task_lists[self.selected];
        selected_list.tasks.remove(selected_list.selected);
        if selected_list.selected == selected_list.tasks.len() {
            selected_list.previous_task();
        }
    }

    fn edit_task(&mut self, task_lists: &mut Vec<TaskList>) {
        if task_lists.is_empty() || task_lists[self.selected].tasks.is_empty() { return; }

        let selected_list = &task_lists[self.selected];
        self.new_task_window.edit_task(selected_list.tasks[selected_list.selected].clone());
    }

    fn new_task(&mut self) {
        self.new_task_window.new_task();
    }

    fn next_tab(&mut self, task_lists: &mut Vec<TaskList>) {
        if task_lists.is_empty() { return; }
        self.selected = (self.selected + 1) % task_lists.len();
    }

    fn previous_tab(&mut self, task_lists: &mut Vec<TaskList>) {
        if task_lists.is_empty() { return; }
        self.selected = (self.selected + task_lists.len() - 1) % task_lists.len();
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, task_lists: &Vec<TaskList>) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
        ]);
        let [task_bar, tasks_area] = vertical.areas(area);

        // Task Bar Rendering
        let mut spans: Vec<Span> = Vec::with_capacity(task_lists.len() + 1);
        let mut highlight_pos: Rect = Rect {x: tasks_area.x + 11, y: tasks_area.y, width: 0, height: 1}; // 11 is length of "Task Lists:"
        spans.push(Span::from("Task Lists:"));

        for (i, list) in task_lists.iter().enumerate() {
            if highlight_pos.width == 0 {
                if i == self.selected { highlight_pos.width = list.name.len() as u16 + 2; }
                else  {highlight_pos.x += list.name.len() as u16 + 2; }
            }

            spans.push(Span::from(format!(" {} ", list.name))
                .style(if i == self.selected {THEME.root_tab_selected} else {THEME.root}));
        }
        Line::from(spans).style(THEME.root).render(task_bar, buf);

        // Task List Rendering
        let horizontal = Layout::horizontal([
            Constraint::Length(4),
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(10),
        ]);

        let tasks_border = Block::bordered()
            .border_style(THEME.task_border)
            .border_type(BorderType::Thick);

        let mut tasks_inner_area = tasks_border.inner(tasks_area);

        tasks_border.render(tasks_area, buf);
        // Render highlight bar
        Block::bordered()
            .style(THEME.task_selected)
            .borders(Borders::TOP)
            .border_type(BorderType::Thick)
            .render(highlight_pos, buf);

        let [_, _, date_area, duration_area] = horizontal.areas(tasks_area);
        Span::styled("Date", THEME.task_title).render(date_area, buf);
        Span::styled("Duration", THEME.task_title).render(duration_area, buf);

        let selected_list = &task_lists[self.selected];
        for (index, task) in selected_list.tasks.iter().enumerate() {
            if !area.intersects(tasks_inner_area) { break; }

            let [mark_area, desc_area, date_area, duration_area] = horizontal.areas(tasks_inner_area);
            Span::styled(
                format!("[{}] ",task.status.get_symbol()),
                if index == selected_list.selected {THEME.task_selected} else {THEME.task}
            ).render(mark_area, buf);
            Span::styled(
                format!(" {} ", task.name),
                if index == selected_list.selected {THEME.task_selected} else {THEME.task}
            ).render(desc_area, buf);
            Span::from(format!(" {} ", disp_md(task.date))).render(date_area, buf);
            Span::from(format!(" {} ", task.duration)).render(duration_area, buf);

            tasks_inner_area = tasks_inner_area.offset(Offset {x: 0, y: 1});

            for (sub_index, sub_task) in task.sub_tasks.iter().enumerate() {
                let [mark_area, desc_area, date_area, duration_area] = horizontal.areas(tasks_inner_area);
                Span::from(format!(" {} ", sub_task.name)).render(desc_area, buf);
                tasks_inner_area = tasks_inner_area.offset(Offset {x: 0, y: 1});
            }
        }

        // Popup Rendering
        if let PopupStatus::InUse = self.new_task_window.status {
            self.new_task_window.render(area, buf);
        }
    }
}

#[derive(Default)]
pub struct CalendarTab {
    pub cal: Calendar,
}

impl CalendarTab {
    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        false
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::horizontal([
            Constraint::Length(24),
            Constraint::Min(50),
        ]);
        let [cal, weekly] = horizontal.areas(area);

        let cal_block = Block::bordered()
            .border_style(THEME.task_border)
            .title_style(THEME.task_title)
            .border_type(BorderType::Thick)
            .title("Monthly View");
        self.cal.render(cal_block.inner(cal).offset(Offset {x: 1, y: 0}), buf);
        cal_block.render(cal, buf);



        Block::bordered()
            .title("Weekly View")
            .border_style(THEME.task_border)
            .title_style(THEME.task_title)
            .border_type(BorderType::Thick)
            .render(weekly, buf);
    }
}

pub struct Options {
    pub delete_on_completion: bool,
    pub error_display_time: u32,
}

pub struct OptionsTab {

}

impl OptionsTab {
    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        false
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, options: &Options) {
        let border = Block::bordered()
            .border_type(BorderType::Thick)
            .style(THEME.task_border);
        Paragraph::new(
            Text::from(vec![
                Line::from(format!(
                    "Delete task on completion: {}",
                    if options.delete_on_completion {'Y'} else {'N'}
                )),
                Line::from(format!(
                    "Error message display time: {} sec", options.error_display_time
                ))
            ])
        )
            .style(THEME.task)
            .block(border)
            .render(area, buf);
    }
}

#[derive(Debug)]
pub struct UserProfile {
    pub name: String,
}

pub struct ProfileTab {

}

impl ProfileTab {
    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        false
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, profile: &UserProfile) {
        Paragraph::new(format!("current_profile: {:?}", profile)).render(area, buf);
    }
}
