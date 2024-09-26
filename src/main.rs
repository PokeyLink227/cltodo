#![allow(unused_variables, dead_code, unused_imports)]

use std::{
    io::{self},
    time::SystemTime,
    mem,
};
use crossterm::{
    event::{self, KeyCode},
};
use ratatui::{
    prelude::*,
    widgets::{
        Block, Paragraph, Widget,
    },
    layout::Flex,
};
use chrono::{NaiveDate};
use crate::{
    tabs::*,
    theme::{THEME},
    popup::*,
};

mod tui;
mod theme;
mod tabs;
mod popup;
mod widgets;

pub enum CommandRequest {
    None,
    SetActive,
}


#[derive(Clone, Copy, PartialEq)]
enum RunningMode {
    Running,
    Exiting,
    Command,
}

enum Tab {
    TaskList,
    Calendar,
    Options,
    Profile,
}

enum Dialogue {
    None,
    Save,
    NewTask,
}

pub struct App {
    mode: RunningMode,
    current_tab: Tab,

    command_str: String,
    error_str: String,
    frames_since_error: Option<u32>,

    task_lists: Vec<TaskList>,
    profile: UserProfile,
    options: Options,

    task_list_tab: TaskListTab,
    calendar_tab: CalendarTab,
    options_tab: OptionsTab,
    profile_tab: ProfileTab,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ]);
        let [title_bar, canvas, bottom_bar] = vertical.areas(area);

        //Block::new().style(THEME.root).render(area, buf);

        self.render_title_bar(title_bar, buf);
        match self.current_tab {
            Tab::TaskList => self.task_list_tab.render(canvas, buf, &self.task_lists),
            Tab::Calendar => self.calendar_tab.render(canvas, buf),
            Tab::Options => self.options_tab.render(canvas, buf, &self.options),
            Tab::Profile => self.profile_tab.render(canvas, buf, &self.profile),
        }
        if self.mode == RunningMode::Command {
            Line::from(vec![Span::from(":"), Span::from(&self.command_str)]).render(bottom_bar, buf);
        } else if let Some(_) = self.frames_since_error {
            Span::from(format!("Error: {}", self.error_str)).style(THEME.command_error).render(bottom_bar, buf);
        } else {
            self.render_bottom_bar(bottom_bar, buf);
        }
    }

}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while self.mode != RunningMode::Exiting {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;

            if let Some(frames) = self.frames_since_error {
                if frames >= self.options.error_display_time * self.options.refresh_rate {
                    self.frames_since_error = None;
                } else {
                    self.frames_since_error = Some(frames + 1);
                }
            }
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                // key holds info about modifiers (shitf, ctrl, alt)
                if key.kind == event::KeyEventKind::Press {
                    if !self.dispatch_input(key.code) {
                        match key.code {
                            KeyCode::Char('q') => self.mode = RunningMode::Exiting,
                            KeyCode::Char('n') => self.next_tab(),
                            KeyCode::Char('N') => self.previous_tab(),
                            KeyCode::Char(':') => {
                                self.mode = RunningMode::Command;
                                self.frames_since_error = None;
                                self.command_str.clear();
                            }
                            _ => {},
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn dispatch_input(&mut self, key: KeyCode) -> bool {
        if self.mode == RunningMode::Command {
            match key {
                KeyCode::Char(c) => self.command_str.push(c),
                KeyCode::Backspace => _ = self.command_str.pop(),
                KeyCode::Enter => {
                    self.mode = RunningMode::Running;
                    self.process_command();
                },
                KeyCode::Esc => {
                    self.mode = RunningMode::Running;
                },
                _ => {},
            }
            true
        } else {
            match self.current_tab {
                Tab::TaskList => self.task_list_tab.handle_input(&mut self.task_lists, key),
                Tab::Calendar => self.calendar_tab.handle_input(key),
                Tab::Options => self.options_tab.handle_input(key),
                Tab::Profile => self.profile_tab.handle_input(key),
            }
        }
    }

    fn process_command(&mut self) {
        let mut parsed_command = self.command_str.split(' ');
        match parsed_command.next().unwrap() {
            "tasks" | "t" => match self.task_list_tab.process_command(parsed_command, &mut self.task_lists) {
                Err(TaskCommandError::UnknownCommand) => {
                    self.frames_since_error = Some(0);
                    self.error_str = format!("Unknown Command: \"{}\"", self.command_str);
                }
                Err(TaskCommandError::InvalidFilePath) => {
                    self.frames_since_error = Some(0);
                    self.error_str = "Invalid File Path".to_string();
                }
                Err(TaskCommandError::InvalidFileFormat) => {
                    self.frames_since_error = Some(0);
                    self.error_str = "Invalid File Format".to_string();
                }
                Ok(CommandRequest::None) => {},
                Ok(CommandRequest::SetActive) => self.current_tab = Tab::TaskList,
            }
            "calendar" | "c"=> self.current_tab = Tab::Calendar,
            "options" | "o" => self.current_tab = Tab::Options,
            "profile" | "p" => self.current_tab = Tab::Profile,
            "quit" | "q" => self.mode = RunningMode::Exiting,
            _ => {
                self.frames_since_error = Some(0);
                self.error_str = format!("Unknown Command: {}", self.command_str);
            }
        }
    }

    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::TaskList => Tab::Calendar,
            Tab::Calendar => Tab::Options,
            Tab::Options => Tab::Profile,
            Tab::Profile => Tab::TaskList,
        }
    }

    fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::TaskList => Tab::Profile,
            Tab::Calendar => Tab::TaskList,
            Tab::Options => Tab::Calendar,
            Tab::Profile => Tab::Options,
        }
    }

    fn render_title_bar(&self, area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(7),
            Constraint::Length(10),
            Constraint::Length(9),
            Constraint::Length(9),
        ]);
        let [app_name, list_tab, calendar_tab, options_tab, profile_tab] = horizontal.areas(area);

        Block::new().style(THEME.root).render(area, buf);
        Paragraph::new("CL-TODO").render(app_name, buf);
        Paragraph::new(" Tasks ").style(if let Tab::TaskList = self.current_tab {THEME.root_tab_selected} else {THEME.root}).render(list_tab, buf);
        Paragraph::new(" Calendar ").style(if let Tab::Calendar = self.current_tab {THEME.root_tab_selected} else {THEME.root}).render(calendar_tab, buf);
        Paragraph::new(" Options ").style(if let Tab::Options = self.current_tab {THEME.root_tab_selected} else {THEME.root}).render(options_tab, buf);
        Paragraph::new(" Profile ").style(if let Tab::Profile = self.current_tab {THEME.root_tab_selected} else {THEME.root}).render(profile_tab, buf);
    }

    /*
        need to only render useable controls for currently selected tab.
        so render common followed by specific controls.
    */
    fn render_bottom_bar(&self, area: Rect, buf: &mut Buffer) {
        let common_keys: [(&'static str, &'static str); 2] = [
            ("Q", "Quit"),
            ("n", "Next Tab"),
        ];

        let other_keys_iter = match self.current_tab {
            Tab::TaskList => self.task_list_tab.controls.iter(),
            _ => [].iter(),
        };

        let spans: Vec<Span> = [common_keys.iter(), other_keys_iter]
            .into_iter()
            .flatten()
            .flat_map(|(key, desc)| {
                let key = Span::from(format!(" {key} ")).style(THEME.key_bind);
                let desc = Span::from(format!(" {desc} ")).style(THEME.key_desc);
                [key, desc]
            })
            .collect();

        Line::from(spans).centered().render(area, buf);
    }
}

fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let mut app = App {
        mode: RunningMode::Running,
        current_tab: Tab::TaskList,
        command_str: String::new(),
        error_str: String::new(),
        frames_since_error: None,
        task_lists: vec![
            TaskList {name: "cl-todo stuff".to_string(), selected: 0, tasks: vec![
                Task {name: "dynamic keybinds bar".to_string(), status: TaskStatus::InProgress, duration: Duration::default(), date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), sub_tasks: vec![
                    Task {name: "reduce rendering time (might use memoization from layout)".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                    Task {name: "reduce rendering time (might use memoization from layout)".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                ]},
                Task {name: "add background to popup".to_string(), status: TaskStatus::Finished, duration: Duration::default(), date: NaiveDate::from_ymd_opt(2024, 7, 8).unwrap(), sub_tasks: Vec::new()},
                Task {name: "reduce rendering time (might use memoization from layout)".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                Task {name: "read/write to file to save list data".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                Task {name: "make dropdown lists for the popup".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                Task {name: "Improve navigation and allowing child windows to capture inputs".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                Task {name: "Task editing".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                Task {name: "add notes section to tasks?".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                Task {name: "add sorting options to task lists".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                Task {name: "add bar highlighting under selected tab instead of connecting to tab".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                Task {name: "add background and possible expansion to tasks".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                Task {name: "move column names to border bar (remove highlighting?)".to_string(), status: TaskStatus::Finished, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                Task {name: "make background coloring better".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                Task {name: "fix imputs escaping current window (not fix but make work better)".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
                Task {name: "bug where switching list while in new task dialogue adds to that list and sometimes crashes".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: NaiveDate::default(), sub_tasks: Vec::new()},
            ]},
            TaskList {name: "Test1 Long tasklist name".to_string(), selected: 0, tasks: Vec::new()},
        ],
        profile: UserProfile {
            name: "Thomas".to_string(),
        },
        options: Options {
            delete_on_completion: false,
            error_display_time: 2,
            refresh_rate: 60,
        },
        task_list_tab: TaskListTab {
            controls: [
                ("J", "Down"),
                ("K", "Up"),
                ("H", "Prev. List"),
                ("L", "Next List"),
                ("M", "Interact"),
            ],
            selected: 0,
            new_task_window: TaskEditorPopup::default(),
        },
        calendar_tab: CalendarTab::default(),
        options_tab: OptionsTab {},
        profile_tab: ProfileTab {}
    };
    app.run(&mut terminal)?;
    tui::restore()
}
