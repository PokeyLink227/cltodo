#![allow(unused_variables, dead_code, unused_imports)]

use crate::{popup::*, tabs::*, theme::THEME, widgets::TextEntry};
use chrono::NaiveDate;
use crossterm::event::{self, KeyCode};
use ratatui::{
    layout::{Flex, Offset},
    prelude::*,
    widgets::{Block, Paragraph, Widget},
};
use std::{
    io::{self},
    mem,
    time::SystemTime,
};

mod popup;
mod tabs;
mod theme;
mod tui;
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

    command_field: TextEntry,
    error_str: String,
    frames_since_error: Option<u32>,

    task_lists: Vec<TaskList>,
    profile: UserProfile,
    options: Options,

    task_list_tab: TaskListTab,
    calendar_tab: CalendarTab,
    options_tab: OptionsTab,
    profile_tab: ProfileTab,
    save_window: ConfirmationPopup,
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

        if self.save_window.status == PopupStatus::InUse {
            self.save_window.render(area, buf);
        }

        if self.mode == RunningMode::Command {
            Line::from(vec![
                Span::from(":"),
                Span::from(self.command_field.get_str()),
            ])
            .render(bottom_bar, buf);
            Span::from("█").render(
                bottom_bar.offset(Offset {
                    x: 1 + self.command_field.get_cursor_pos() as i32,
                    y: 0,
                }),
                buf,
            );
        } else if let Some(_) = self.frames_since_error {
            Span::from(format!("Error: {}", self.error_str))
                .style(THEME.command_error)
                .render(bottom_bar, buf);
        } else {
            self.render_bottom_bar(bottom_bar, buf);
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        // initialization
        self.command_field.set_text("t load".to_string());
        self.process_command();

        // main loop
        while self.mode != RunningMode::Exiting {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;

            // command error timer update
            if let Some(frames) = self.frames_since_error {
                if frames >= self.options.error_display_time * self.options.refresh_rate {
                    self.frames_since_error = None;
                } else {
                    self.frames_since_error = Some(frames + 1);
                }
            }

            // popup handler
            match self.save_window.status {
                PopupStatus::Closed | PopupStatus::InUse => {}
                PopupStatus::Canceled => {
                    self.save_window.close();
                }
                PopupStatus::Confirmed => {
                    self.save_window.close();
                    self.mode = RunningMode::Exiting;
                    if self.save_window.decision() {
                        self.command_field.set_text("t save".to_string());
                        self.process_command();
                    }
                }
            }
        }

        // clean up

        // report no errors
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
                            KeyCode::Char('q') => self.try_quit(),
                            KeyCode::Char('n') => self.next_tab(),
                            KeyCode::Char('N') => self.previous_tab(),
                            KeyCode::Char(':') => {
                                self.mode = RunningMode::Command;
                                self.frames_since_error = None;
                                self.command_field.clear();
                            }
                            _ => {}
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
                KeyCode::Char(c) => self.command_field.insert(c),
                KeyCode::Backspace => self.command_field.remove(),
                KeyCode::Enter => {
                    self.mode = RunningMode::Running;
                    self.process_command();
                    self.command_field.move_cursor_home();
                }
                KeyCode::Esc => {
                    self.mode = RunningMode::Running;
                    self.command_field.move_cursor_home();
                }
                KeyCode::Left => self.command_field.move_cursor_left(),
                KeyCode::Right => self.command_field.move_cursor_right(),
                _ => {}
            }
            true
        } else if self.save_window.status == PopupStatus::InUse {
            self.save_window.handle_input(key)
        } else {
            match self.current_tab {
                Tab::TaskList => self.task_list_tab.handle_input(&mut self.task_lists, key),
                Tab::Calendar => self.calendar_tab.handle_input(key),
                Tab::Options => self.options_tab.handle_input(key),
                Tab::Profile => self.profile_tab.handle_input(key),
            }
        }
    }

    // currently doesnt support arguments with spaces included
    fn process_command(&mut self) {
        let mut parsed_command = self.command_field.get_str().split(' ');
        match parsed_command.next().unwrap() {
            "tasks" | "t" => match self
                .task_list_tab
                .process_command(parsed_command, &mut self.task_lists)
            {
                Err(TaskCommandError::UnknownCommand) => self.post_error(format!(
                    "Unknown Command: \"{}\"",
                    self.command_field.get_str()
                )),
                Err(TaskCommandError::InvalidFilePath) => {
                    self.post_error("Invalid File Path".to_string())
                }
                Err(TaskCommandError::InvalidFileFormat) => {
                    self.post_error("Invalid File Format".to_string())
                }
                Err(TaskCommandError::MissingField) => self.post_error("Missing Field".to_string()),
                Err(TaskCommandError::NotANumber) => self.post_error("Not A Number".to_string()),
                Ok(CommandRequest::None) => {}
                Ok(CommandRequest::SetActive) => self.current_tab = Tab::TaskList,
            },
            "calendar" | "c" => self.current_tab = Tab::Calendar,
            "options" | "o" => self.current_tab = Tab::Options,
            "profile" | "p" => self.current_tab = Tab::Profile,
            "quit" | "q" => self.try_quit(),
            "quit!" | "q!" => self.force_quit(),
            _ => self.post_error(format!("Unknown Command: {}", self.command_field.get_str())),
        }
    }

    fn post_error(&mut self, err_str: String) {
        self.frames_since_error = Some(0);
        self.error_str = err_str;
    }

    fn force_quit(&mut self) {
        self.mode = RunningMode::Exiting;
    }

    fn try_quit(&mut self) {
        self.save_window.show();
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
        Paragraph::new("FrogPad").render(app_name, buf);
        Paragraph::new(" Tasks ")
            .style(if let Tab::TaskList = self.current_tab {
                THEME.root_tab_selected
            } else {
                THEME.root
            })
            .render(list_tab, buf);
        Paragraph::new(" Calendar ")
            .style(if let Tab::Calendar = self.current_tab {
                THEME.root_tab_selected
            } else {
                THEME.root
            })
            .render(calendar_tab, buf);
        Paragraph::new(" Options ")
            .style(if let Tab::Options = self.current_tab {
                THEME.root_tab_selected
            } else {
                THEME.root
            })
            .render(options_tab, buf);
        Paragraph::new(" Profile ")
            .style(if let Tab::Profile = self.current_tab {
                THEME.root_tab_selected
            } else {
                THEME.root
            })
            .render(profile_tab, buf);
    }

    /*
        need to only render useable controls for currently selected tab.
        so render common followed by specific controls.
    */
    fn render_bottom_bar(&self, area: Rect, buf: &mut Buffer) {
        let common_keys: [(&'static str, &'static str); 2] = [("Q", "Quit"), ("n", "Next Tab")];

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
        command_field: TextEntry::default(),
        error_str: String::new(),
        frames_since_error: None,
        task_lists: Vec::new(),
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
            delete_conf_window: ConfirmationPopup::new(
                "Confirm delete".to_string(),
                "Are you sure you want to delete?".to_string(),
            ),
            new_tasklist_window: TextEntryPopup::new("Enter TaskList Name".to_string(), 3),
            selected_sub_task: 0,
        },
        calendar_tab: CalendarTab::default(),
        options_tab: OptionsTab {},
        profile_tab: ProfileTab {},
        save_window: ConfirmationPopup::new(
            "Confirm Save".to_string(),
            "There is unsaved work. Save now?".to_string(),
        ),
    };
    app.run(&mut terminal)?;
    tui::restore()
}
