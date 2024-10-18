use crate::{
    popup::{
        ConfirmationField, ConfirmationPopup, PopupStatus, TaskEditorPopup, TaskSource,
        TextEntryPopup,
    },
    theme::THEME,
    widgets::Calendar,
    CommandRequest,
};
use chrono::{Datelike, NaiveDate, Weekday};
use crossterm::event::KeyCode;
use ratatui::{layout::Offset, prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::prelude::*, str::Split};

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

pub fn disp_md(date: NaiveDate) -> String {
    format!(
        "{} {:02}",
        match date.month() {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "ERR",
        },
        date.day(),
    )
}

pub enum TaskCommandError {
    UnknownCommand,
    InvalidFilePath,
    InvalidFileFormat,
    NotANumber,
    MissingField,
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

    #[serde(skip)]
    pub expanded: bool,
}

#[derive(Deserialize, Serialize)]
pub struct TaskList {
    pub name: String,
    pub selected: usize,
    pub tasks: Vec<Task>,
}

impl TaskList {
    pub fn new(new_name: String, tasks_new: Option<Vec<Task>>) -> Self {
        TaskList {
            name: new_name,
            selected: 0,
            tasks: tasks_new.or(Some(Vec::new())).unwrap(),
        }
    }

    fn next_task(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.tasks.len();
    }

    fn previous_task(&mut self) {
        if self.tasks.is_empty() {
            return;
        }
        self.selected = (self.selected + self.tasks.len() - 1) % self.tasks.len();
    }
}

pub struct TaskListTab {
    pub controls: [(&'static str, &'static str); 5],
    pub selected: usize,

    pub new_task_window: TaskEditorPopup,
    pub delete_conf_window: ConfirmationPopup,
    pub new_tasklist_window: TextEntryPopup,

    pub selected_sub_task: usize,
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
                    TaskSource::Existing => {
                        let task = &mut selected_list.tasks[selected_list.selected];
                        if task.sub_tasks.len() > 0 && task.expanded && self.selected_sub_task != 0
                        {
                            task.sub_tasks[self.selected_sub_task - 1] =
                                self.new_task_window.take_task();
                        } else {
                            *task = self.new_task_window.take_task();
                        }
                    }
                }
                self.new_task_window.status = PopupStatus::Closed;
            }
        } else if self.delete_conf_window.status == PopupStatus::InUse {
            input_captured = self.delete_conf_window.handle_input(key);

            match self.delete_conf_window.status {
                PopupStatus::InUse | PopupStatus::Closed => {}
                PopupStatus::Confirmed => {
                    if self.delete_conf_window.decision() {
                        self.delete_task(task_lists);
                    }
                    self.delete_conf_window.close();
                }
                PopupStatus::Canceled => {
                    self.delete_conf_window.close();
                }
            }
        } else if self.new_tasklist_window.status == PopupStatus::InUse {
            input_captured = self.new_tasklist_window.handle_input(key);

            match self.new_tasklist_window.status {
                PopupStatus::InUse | PopupStatus::Closed => {}
                PopupStatus::Canceled => self.new_tasklist_window.reset(),
                PopupStatus::Confirmed => task_lists.push(TaskList {
                    name: self.new_tasklist_window.take(),
                    selected: 0,
                    tasks: Vec::new(),
                }),
            }
        } else {
            match key {
                KeyCode::Char('h') => self.previous_tab(task_lists),
                KeyCode::Char('l') => self.next_tab(task_lists),
                KeyCode::Char('k') => {
                    if selected_list.tasks[selected_list.selected].expanded {
                        if self.selected_sub_task == 0 {
                            selected_list.previous_task();
                            if selected_list.tasks[selected_list.selected].expanded {
                                self.selected_sub_task =
                                    selected_list.tasks[selected_list.selected].sub_tasks.len();
                            }
                        } else {
                            self.selected_sub_task -= 1;
                        }
                    } else {
                        selected_list.previous_task();
                        if selected_list.tasks[selected_list.selected].expanded {
                            self.selected_sub_task =
                                selected_list.tasks[selected_list.selected].sub_tasks.len();
                        }
                    }
                }
                KeyCode::Char('j') => {
                    if selected_list.tasks[selected_list.selected].expanded {
                        self.selected_sub_task += 1;
                        if self.selected_sub_task
                            > selected_list.tasks[selected_list.selected].sub_tasks.len()
                        {
                            self.selected_sub_task = 0;
                            selected_list.next_task();
                        }
                    } else {
                        selected_list.next_task();
                    }
                }
                KeyCode::Char('a') => self.new_task(),
                KeyCode::Char('e') => self.edit_task(task_lists),
                KeyCode::Char('m') => self.mark_task(task_lists),
                KeyCode::Char('d') => self.try_delete_task(task_lists),
                KeyCode::Right => {
                    let task = &mut selected_list.tasks[selected_list.selected];
                    if task.expanded {
                        task.expanded = false;
                        self.selected_sub_task = 0;
                    } else {
                        task.expanded = true;
                    }
                }
                //KeyCode::Char('s') => self.save_data(task_lists),
                //KeyCode::Char('S') => self.load_data(task_lists),
                _ => input_captured = false,
            }
        }

        input_captured
    }

    pub fn process_command(
        &mut self,
        mut command: Split<char>,
        task_lists: &mut Vec<TaskList>,
    ) -> Result<CommandRequest, TaskCommandError> {
        match command.next() {
            Some("new") => {
                self.new_task();
                Ok(CommandRequest::None)
            }
            Some("newlist") => {
                self.new_task_list();
                Ok(CommandRequest::None)
            }
            Some("save") => match command.next() {
                Some(filename) => self.save_data(filename, task_lists),
                None => self.save_data("list.json", task_lists),
            },
            Some("load") => match command.next() {
                Some(filename) => self.load_data(filename, task_lists),
                None => self.load_data("list.json", task_lists),
            },
            Some("import") => match command.next() {
                Some(filename) => self.load_list(filename, task_lists),
                None => Err(TaskCommandError::MissingField),
            },
            Some("export") => match command.next() {
                Some(num_str) => {
                    let list_index = num_str
                        .parse::<usize>()
                        .or(Err(TaskCommandError::NotANumber))?;
                    self.save_list(&task_lists[list_index])
                }
                None => Err(TaskCommandError::MissingField),
            },
            None => Ok(CommandRequest::SetActive),
            Some(_) => Err(TaskCommandError::UnknownCommand),
        }
    }

    fn load_data(
        &mut self,
        filename: &str,
        task_lists: &mut Vec<TaskList>,
    ) -> Result<CommandRequest, TaskCommandError> {
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(_) => return Err(TaskCommandError::InvalidFilePath),
        };
        let mut data = Vec::new();
        _ = file.read_to_end(&mut data).unwrap();
        let temp: Vec<TaskList> = match serde_json::from_slice(&data) {
            Ok(v) => v,
            Err(_) => return Err(TaskCommandError::InvalidFileFormat),
        };
        *task_lists = temp;
        Ok(CommandRequest::None)
    }

    /*
        Loads a list from the disk and adds it to the current list of tasklists
    */
    fn load_list(
        &mut self,
        filename: &str,
        task_lists: &mut Vec<TaskList>,
    ) -> Result<CommandRequest, TaskCommandError> {
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(_) => return Err(TaskCommandError::InvalidFilePath),
        };
        let mut data = Vec::new();
        _ = file.read_to_end(&mut data).unwrap();
        let temp: TaskList = match serde_json::from_slice(&data) {
            Ok(v) => v,
            Err(_) => return Err(TaskCommandError::InvalidFileFormat),
        };
        task_lists.push(temp);
        Ok(CommandRequest::None)
    }

    fn save_data(
        &mut self,
        filename: &str,
        task_lists: &mut Vec<TaskList>,
    ) -> Result<CommandRequest, TaskCommandError> {
        let mut file = match File::create(filename) {
            Ok(f) => f,
            Err(_) => return Err(TaskCommandError::InvalidFilePath),
        };
        let out = serde_json::to_vec(&task_lists).unwrap();
        _ = file.write_all(&out).unwrap();
        Ok(CommandRequest::None)
    }

    fn save_list(&mut self, task_list: &TaskList) -> Result<CommandRequest, TaskCommandError> {
        let mut file = match File::create(format!("{}.json", task_list.name)) {
            Ok(f) => f,
            Err(_) => return Err(TaskCommandError::InvalidFilePath),
        };
        let out = serde_json::to_vec(task_list).unwrap();
        _ = file.write_all(&out).unwrap();
        Ok(CommandRequest::None)
    }

    fn mark_task(&mut self, task_lists: &mut Vec<TaskList>) {
        if task_lists.is_empty() || task_lists[self.selected].tasks.is_empty() {
            return;
        }

        let list = &mut task_lists[self.selected];
        let task = &mut list.tasks[list.selected];
        if task.sub_tasks.len() > 0 && task.expanded && self.selected_sub_task != 0 {
            task.sub_tasks[self.selected_sub_task - 1]
                .status
                .cycle_next();
        } else {
            task.status.cycle_next();
        }
    }

    fn try_delete_task(&mut self, task_lists: &mut Vec<TaskList>) {
        if task_lists.is_empty() || task_lists[self.selected].tasks.is_empty() {
            return;
        }

        self.delete_conf_window.show();
    }

    fn delete_task(&mut self, task_lists: &mut Vec<TaskList>) {
        if task_lists.is_empty() || task_lists[self.selected].tasks.is_empty() {
            return;
        }

        let selected_list = &mut task_lists[self.selected];
        let task = &mut selected_list.tasks[selected_list.selected];
        if task.sub_tasks.len() > 0 && task.expanded && self.selected_sub_task != 0 {
            task.sub_tasks.remove(self.selected_sub_task - 1);
            if self.selected_sub_task > task.sub_tasks.len() {
                self.selected_sub_task -= 1;
            }
        } else {
            selected_list.tasks.remove(selected_list.selected);
            if selected_list.selected == selected_list.tasks.len() {
                selected_list.previous_task();
            }
        }
    }

    fn edit_task(&mut self, task_lists: &mut Vec<TaskList>) {
        if task_lists.is_empty() || task_lists[self.selected].tasks.is_empty() {
            return;
        }

        let selected_list = &task_lists[self.selected];
        let task = &selected_list.tasks[selected_list.selected];

        if task.sub_tasks.len() > 0 && task.expanded && self.selected_sub_task != 0 {
            self.new_task_window
                .edit_task(task.sub_tasks[self.selected_sub_task - 1].clone());
        } else {
            self.new_task_window.edit_task(task.clone());
        }
    }

    fn new_task(&mut self) {
        self.new_task_window.new_task();
    }

    fn new_task_list(&mut self) {
        self.new_tasklist_window.show();
    }

    fn next_tab(&mut self, task_lists: &mut Vec<TaskList>) {
        if task_lists.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % task_lists.len();
    }

    fn previous_tab(&mut self, task_lists: &mut Vec<TaskList>) {
        if task_lists.is_empty() {
            return;
        }
        self.selected = (self.selected + task_lists.len() - 1) % task_lists.len();
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, task_lists: &Vec<TaskList>) {
        let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
        let [task_bar, tasks_area] = vertical.areas(area);

        // Task Bar Rendering
        let mut spans: Vec<Span> = Vec::with_capacity(task_lists.len() + 1);
        let mut highlight_pos: Rect = Rect {
            x: tasks_area.x + 11,
            y: tasks_area.y,
            width: 0,
            height: 1,
        }; // 11 is length of "Task Lists:"
        spans.push(Span::from("Task Lists:"));

        for (i, list) in task_lists.iter().enumerate() {
            if highlight_pos.width == 0 {
                if i == self.selected {
                    highlight_pos.width = list.name.len() as u16 + 2;
                } else {
                    highlight_pos.x += list.name.len() as u16 + 2;
                }
            }

            spans.push(
                Span::from(format!(" {} ", list.name)).style(if i == self.selected {
                    THEME.root_tab_selected
                } else {
                    THEME.root
                }),
            );
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
            .border_type(BorderType::Rounded);

        let mut tasks_inner_area = tasks_border.inner(tasks_area);

        tasks_border.render(tasks_area, buf);
        // Render highlight bar
        Block::bordered()
            .style(THEME.task_selected)
            .borders(Borders::TOP)
            .border_type(BorderType::Rounded)
            .render(highlight_pos, buf);

        let [_, _, date_area, duration_area] = horizontal.areas(tasks_area);
        Span::styled("Date", THEME.task_title).render(date_area, buf);
        Span::styled("Duration", THEME.task_title).render(duration_area, buf);

        let selected_list = &task_lists[self.selected];
        for (index, task) in selected_list.tasks.iter().enumerate() {
            if !area.intersects(tasks_inner_area) {
                break;
            }

            let [mark_area, desc_area, date_area, duration_area] =
                horizontal.areas(tasks_inner_area);
            Span::styled(
                format!("[{}] ", task.status.get_symbol()),
                if index == selected_list.selected {
                    THEME.task_selected
                } else {
                    THEME.task
                },
            )
            .render(mark_area, buf);
            Span::styled(
                format!(" {} ", task.name),
                if index == selected_list.selected && self.selected_sub_task == 0 {
                    THEME.task_selected
                } else {
                    THEME.task
                },
            )
            .render(desc_area, buf);
            Span::from(format!(" {} ", disp_md(task.date))).render(date_area, buf);
            Span::from(format!(" {} ", task.duration)).render(duration_area, buf);

            tasks_inner_area = tasks_inner_area.offset(Offset { x: 0, y: 1 });

            if task.expanded {
                let horizontal = Layout::horizontal([
                    Constraint::Length(5),
                    Constraint::Length(3),
                    Constraint::Min(20),
                    Constraint::Length(8),
                    Constraint::Length(10),
                ]);

                for (sub_index, sub_task) in task.sub_tasks.iter().enumerate() {
                    let [tree_area, mark_area, desc_area, date_area, duration_area] =
                        horizontal.areas(tasks_inner_area);
                    let style = if index == selected_list.selected
                        && self.selected_sub_task == sub_index + 1
                    {
                        THEME.task_selected
                    } else {
                        THEME.task
                    };
                    Line::from(vec![
                        Span::from(if sub_index == task.sub_tasks.len() - 1 {
                            " └─"
                        } else {
                            " ├─"
                        })
                        .style(style),
                        Span::from(if sub_index == task.sub_tasks.len() - 1 {
                            "─"
                        } else {
                            "─"
                        })
                        .style(style),
                    ])
                    .render(tree_area, buf);
                    Span::styled(format!("[{}]", sub_task.status.get_symbol()), style)
                        .render(mark_area, buf);
                    Span::from(format!(" {} ", sub_task.name))
                        .style(style)
                        .render(desc_area, buf);
                    tasks_inner_area = tasks_inner_area.offset(Offset { x: 0, y: 1 });
                }
            }
        }

        // Popup Rendering
        if PopupStatus::InUse == self.new_task_window.status {
            self.new_task_window.render(area, buf);
        } else if self.delete_conf_window.status == PopupStatus::InUse {
            self.delete_conf_window.render(area, buf);
        } else if self.new_tasklist_window.status == PopupStatus::InUse {
            self.new_tasklist_window.render(area, buf);
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
        let horizontal = Layout::horizontal([Constraint::Length(24), Constraint::Min(50)]);
        let [cal, weekly] = horizontal.areas(area);

        let cal_block = Block::bordered()
            .border_style(THEME.task_border)
            .title_style(THEME.task_title)
            .border_type(BorderType::Rounded)
            .title("Monthly View");
        self.cal
            .render(cal_block.inner(cal).offset(Offset { x: 1, y: 0 }), buf);
        cal_block.render(cal, buf);

        Block::bordered()
            .title("Weekly View")
            .border_style(THEME.task_border)
            .title_style(THEME.task_title)
            .border_type(BorderType::Rounded)
            .render(weekly, buf);
    }
}

pub struct Options {
    pub delete_on_completion: bool,
    pub error_display_time: u32,
    pub refresh_rate: u32,
}

pub struct OptionsTab {}

impl OptionsTab {
    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        false
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, options: &Options) {
        let border = Block::bordered()
            .border_type(BorderType::Rounded)
            .style(THEME.task_border);
        Paragraph::new(Text::from(vec![
            Line::from(format!(
                "Delete task on completion: {}",
                if options.delete_on_completion {
                    'Y'
                } else {
                    'N'
                }
            )),
            Line::from(format!(
                "Error message display time: {} sec",
                options.error_display_time
            )),
        ]))
        .style(THEME.task)
        .block(border)
        .render(area, buf);
    }
}

#[derive(Debug)]
pub struct UserProfile {
    pub name: String,
}

pub struct ProfileTab {}

impl ProfileTab {
    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        false
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, profile: &UserProfile) {
        Paragraph::new(format!("current_profile: {:?}", profile)).render(area, buf);
    }
}
