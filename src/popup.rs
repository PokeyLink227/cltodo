use ratatui::{
    prelude::*,
    widgets::*,
    layout::Flex,
};
use crossterm::event::{KeyCode};
use crate::{
    theme::{THEME},
    tabs::{TaskListTab, TaskList, Task, TaskStatus},
};

#[derive(Default, PartialEq)]
pub enum PopupStatus {
    InUse,
    Canceled,
    Confirmed,
    #[default]
    Closed,
}

#[derive(Default, PartialEq)]
enum Mode {
    Editing,
    #[default]
    Navigating,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Editing => "Edit",
            Self::Navigating => "Nav",
        })
    }
}

#[derive(Default, PartialEq)]
enum TaskEditorField {
    #[default]
    Description,
    Status,
    Date,
    Duration,
    Cancel,
    Confirm,
}

#[derive(Default, PartialEq)]
pub enum TaskSource {
    #[default]
    NewTask,
    ExistingTask,
}

// rename to TaskEditor
// create method to export/consume new task data
#[derive(Default)]
pub struct TaskEditorPopup {
    pub status: PopupStatus,
    pub task_source: TaskSource,

    task: Task,
    mode: Mode,
    selected_field: TaskEditorField,
}

impl TaskEditorPopup {
    pub fn handle_input(&mut self, key: KeyCode) {
        match self.selected_field {
            TaskEditorField::Description => match self.mode {
                Mode::Editing => match key {
                    KeyCode::Char(c) => self.task.name.push(c),
                    KeyCode::Backspace => { self.task.name.pop(); },
                    KeyCode::Enter => self.mode = Mode::Navigating,
                    _ => {},
                },
                Mode::Navigating => match key {
                    KeyCode::Char('e') | KeyCode::Char('/') | KeyCode::Enter => self.mode = Mode::Editing,
                    KeyCode::Char('j') => self.selected_field = TaskEditorField::Cancel,
                    KeyCode::Char('l') => self.selected_field = TaskEditorField::Status,
                    _ => {},
                },
            },
            TaskEditorField::Cancel => match key {
                KeyCode::Enter | KeyCode::Char('e') => self.status = PopupStatus::Canceled,
                KeyCode::Char('k') => self.selected_field = TaskEditorField::Description,
                KeyCode::Char('l') => self.selected_field = TaskEditorField::Confirm,
                _ => {},
            },
            TaskEditorField::Confirm => match key {
                KeyCode::Enter | KeyCode::Char('e') => self.status = PopupStatus::Confirmed,
                KeyCode::Char('h') => self.selected_field = TaskEditorField::Cancel,
                KeyCode::Char('k') => self.selected_field = TaskEditorField::Description,
                _ => {},
            },
            TaskEditorField::Status => match self.mode {
                Mode::Editing => match key {
                    KeyCode::Enter => self.mode = Mode::Editing,
                    _ => {},
                },
                Mode::Navigating => match key {
                    KeyCode::Char('e') => self.mode = Mode::Editing,
                    KeyCode::Char('l') => self.selected_field = TaskEditorField::Date,
                    _ => {},
                },
            },
            TaskEditorField::Date => match key {
                KeyCode::Char('l') => self.selected_field = TaskEditorField::Duration,
                _ => {},
            },
            TaskEditorField::Duration => match key {
                KeyCode::Char('l') => self.selected_field = TaskEditorField::Cancel,
                _ => {},
            },
        }
    }

    pub fn take_task(&mut self) -> Task {
        std::mem::take(&mut self.task)
    }

    pub fn edit_task(&mut self, task: Task) {
        self.task = task;
        self.status = PopupStatus::InUse;
        self.task_source = TaskSource::ExistingTask;
    }

    pub fn new_task(&mut self) {
        self.status = PopupStatus::InUse;
        self.selected_field = TaskEditorField::Description;
        self.mode = Mode::Editing;
        self.task_source = TaskSource::NewTask;
    }
}

impl Widget for &TaskEditorPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([5]).flex(Flex::Center);
        let horizontal = Layout::horizontal([60]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);


        let window = Block::bordered()
            .style(THEME.popup)
            .border_style(THEME.popup)
            .title(Line::from("New Task"))
            .title_bottom(format!(" {} ", self.mode));

        let win_area = window.inner(area);
        Clear.render(win_area, buf);
        window.render(area, buf);

        let vert = Layout::vertical([1, 1, 1]);
        let [top_area, mid_area, bot_area] = vert.areas(win_area);

        let bot_horiz = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(10),
            Constraint::Length(11),
        ]);
        let [_, cancel_area, quit_area] = bot_horiz.areas(bot_area);
        Paragraph::new(" [Cancel] ")
            .style(if self.selected_field == TaskEditorField::Cancel {THEME.popup_selected} else {THEME.popup})
            .render(cancel_area, buf);
        Paragraph::new(" [Confirm] ")
            .style(if self.selected_field == TaskEditorField::Confirm {THEME.popup_selected} else {THEME.popup})
            .render(quit_area, buf);

        let mid_horiz = Layout::horizontal([
            Constraint::Length(10),
            Constraint::Length(13),
            Constraint::Length(14),
        ]);
        let [status_area, date_area, duration_area] = mid_horiz.areas(mid_area);
        Span::styled(
            format!("Status: 0"),
            if self.selected_field == TaskEditorField::Status {THEME.popup_selected} else {THEME.popup}
        )
            .render(status_area, buf);
        Span::styled(
            format!("Date: {}", self.task.date),
            if self.selected_field == TaskEditorField::Date {THEME.popup_selected} else {THEME.popup}
        )
            .render(date_area, buf);
        Span::styled(
            format!("Dur: {}", self.task.duration),
            if self.selected_field == TaskEditorField::Duration {THEME.popup_selected} else {THEME.popup}
        )
            .render(duration_area, buf);

        //let text_entry = Paragraph::new(self.text.as_str()).wrap(Wrap {trim: true});
        let top_horiz = Layout::horizontal([
            Constraint::Min(0),
        ]);
        let [text_area] = top_horiz.areas(top_area);
        Line::from(vec![
            Span::from("Desc: "),
            Span::from(self.task.name.as_str()),
        ])
            .style(if self.selected_field == TaskEditorField::Description {THEME.popup_selected} else {THEME.popup})
            .render(text_area, buf);

    }

}
