#![allow(unused_variables, dead_code, unused_imports)]

use std::io::{self};
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
use crate::{
    tabs::*,
    theme::{THEME},
    popup::*,
};

mod tui;
mod theme;
mod tabs;
mod popup;


enum RunningMode {
    Running,
    Exiting,
}

enum Tab {
    TaskList,
    Calender,
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

    task_list_tab: TaskListTab,
    calender_tab: CalenderTab,
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
            Tab::TaskList => self.task_list_tab.render(canvas, buf),
            Tab::Calender => self.calender_tab.render(canvas, buf),
            Tab::Options => self.options_tab.render(canvas, buf),
            Tab::Profile => self.profile_tab.render(canvas, buf),
        }
        self.render_bottom_bar(bottom_bar, buf);
    }

}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while let RunningMode::Running = self.mode {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
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
                            KeyCode::Tab => self.next_tab(),
                            KeyCode::BackTab => self.previous_tab(),
                            _ => {},
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn dispatch_input(&mut self, key: KeyCode) -> bool {
        match self.current_tab {
            Tab::TaskList => self.task_list_tab.handle_input(key),
            Tab::Calender => self.calender_tab.handle_input(key),
            Tab::Options => self.options_tab.handle_input(key),
            Tab::Profile => self.profile_tab.handle_input(key),
        }
    }

    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::TaskList => Tab::Calender,
            Tab::Calender => Tab::Options,
            Tab::Options => Tab::Profile,
            Tab::Profile => Tab::TaskList,
        }
    }

    fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::TaskList => Tab::Profile,
            Tab::Calender => Tab::TaskList,
            Tab::Options => Tab::Calender,
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
        let [app_name, list_tab, calender_tab, options_tab, profile_tab] = horizontal.areas(area);

        Block::new().style(THEME.root).render(area, buf);
        Paragraph::new("CL-TODO").render(app_name, buf);
        Paragraph::new(" Tasks ").style(if let Tab::TaskList = self.current_tab {THEME.root_tab_selected} else {THEME.root}).render(list_tab, buf);
        Paragraph::new(" Calender ").style(if let Tab::Calender = self.current_tab {THEME.root_tab_selected} else {THEME.root}).render(calender_tab, buf);
        Paragraph::new(" Options ").style(if let Tab::Options = self.current_tab {THEME.root_tab_selected} else {THEME.root}).render(options_tab, buf);
        Paragraph::new(" Profile ").style(if let Tab::Profile = self.current_tab {THEME.root_tab_selected} else {THEME.root}).render(profile_tab, buf);
    }

    /*
        need to only render useable controls for currently selected tab.
        so render common followed by specific controls.
    */
    fn render_bottom_bar(&self, area: Rect, buf: &mut Buffer) {
        let common_keys: [(&'static str, &'static str); 4] = [
            ("Q/Esc", "Quit"),
            ("Tab", "Switch Tab"),
            ("J", "Down"),
            ("K", "Up"),
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
        task_list_tab: TaskListTab {
            controls: [
                ("H", "Prev. List"),
                ("L", "Next List"),
                ("E", "Interact"),
            ],
            selected: 0,
            task_lists: vec![
                TaskList {name: "cl-todo stuff".to_string(), selected: 0, tasks: vec![
                    Task {name: "dynamic keybinds bar".to_string(), status: TaskStatus::InProgress, duration: Duration::default(), date: Date {month: 1, day: 15}},
                    Task {name: "add background to popup".to_string(), status: TaskStatus::Finished, duration: Duration::default(), date: Date {month: 7, day: 8}},
                    Task {name: "reduce rendering time (might use memoization from layout)".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: Date::default()},
                    Task {name: "read/write to file to save list data".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: Date::default()},
                    Task {name: "make dropdown lists for the popup".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: Date::default()},
                    Task {name: "Improve navigation and allowing child windows to capture inputs".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: Date::default()},
                    Task {name: "Task editing".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: Date::default()},
                    Task {name: "add notes section to tasks?".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: Date::default()},
                    Task {name: "add sorting options to task lists".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: Date::default()},
                    Task {name: "add bar highlighting under selected tab instead of connecting to tab".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: Date::default()},
                    Task {name: "add background and possible expansion to tasks".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: Date::default()},
                    Task {name: "move column names to border bar (remove highlighting?)".to_string(), status: TaskStatus::Finished, duration: Duration::default(), date: Date::default()},
                    Task {name: "make background coloring better".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: Date::default()},
                    Task {name: "fix imputs escaping current window (not fix but make work better)".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: Date::default()},
                    Task {name: "bug where switching list while in new task dialogue adds to that list and sometimes crashes".to_string(), status: TaskStatus::NotStarted, duration: Duration::default(), date: Date::default()},
                ]},
                TaskList {name: "Test1 Long tasklist name".to_string(), selected: 0, tasks: Vec::new()},
            ],
            new_task_window: TaskEditorPopup::default(),
        },
        calender_tab: CalenderTab {},
        options_tab: OptionsTab {},
        profile_tab: ProfileTab {}
    };
    app.run(&mut terminal)?;
    tui::restore()
}
