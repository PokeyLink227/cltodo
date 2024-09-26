use ratatui::{
    prelude::*,
    widgets::*,
    layout::Flex,
};
use crossterm::event::{KeyCode};
use crate::{
    theme::{THEME},
    tabs::{Task, TaskStatus},
};

#[derive(Default, PartialEq)]
pub enum PopupStatus {
    InUse,
    Canceled,
    Confirmed,
    #[default]
    Closed,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum TaskEditorField {
    #[default]
    Description,
    Status,
    Date,
    Duration,
    Cancel,
    Confirm,
}

impl TaskEditorField {
    fn next(&mut self) {
        *self = match self {
            TaskEditorField::Description => TaskEditorField::Status,
            TaskEditorField::Status => TaskEditorField::Date,
            TaskEditorField::Date => TaskEditorField::Duration,
            TaskEditorField::Duration => TaskEditorField::Cancel,
            TaskEditorField::Cancel => TaskEditorField::Confirm,
            TaskEditorField::Confirm => TaskEditorField::Description,
        }
    }

    fn previous(&mut self) {
        *self = match self {
            TaskEditorField::Description => TaskEditorField::Confirm,
            TaskEditorField::Status => TaskEditorField::Description,
            TaskEditorField::Date => TaskEditorField::Status,
            TaskEditorField::Duration => TaskEditorField::Date,
            TaskEditorField::Cancel => TaskEditorField::Duration,
            TaskEditorField::Confirm => TaskEditorField::Cancel,
        }
    }
}

#[derive(Default, PartialEq)]
pub enum TaskSource {
    #[default]
    New,
    Existing,
}

// rename to TaskEditor
// create method to export/consume new task data
#[derive(Default)]
pub struct TaskEditorPopup {
    pub status: PopupStatus,
    pub task_source: TaskSource,

    task: Task,
    selected_field: TaskEditorField,
    field_pos: u16,
}

impl TaskEditorPopup {
    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        let mut input_captured = true;

        match key {
            KeyCode::Enter => self.status = PopupStatus::Confirmed,
            KeyCode::Esc => self.status = PopupStatus::Closed,
            KeyCode::Tab => self.selected_field.next(),
            KeyCode::BackTab => self.selected_field.previous(),
            _ => {}
        }

        match self.selected_field {
            TaskEditorField::Description => match key {
                KeyCode::Char(c) => self.task.name.push(c),
                KeyCode::Backspace => { self.task.name.pop(); },
                _ => input_captured = false,
            },
            TaskEditorField::Cancel => match key {
                KeyCode::Char('e') => self.status = PopupStatus::Canceled,
                _ => input_captured = false,
            },
            TaskEditorField::Confirm => match key {
                KeyCode::Char('e') => self.status = PopupStatus::Confirmed,
                _ => input_captured = false,
            },
            TaskEditorField::Status => match key {
                KeyCode::Char('1') => self.task.status = TaskStatus::NotStarted,
                KeyCode::Char('2') => self.task.status = TaskStatus::InProgress,
                KeyCode::Char('3') => self.task.status = TaskStatus::Finished,
                _ => input_captured = false,
            },
            TaskEditorField::Date => match key {
                KeyCode::Char(c) if c >= '0' && c <= '9' => {},
                _ => input_captured = false,
            },
            TaskEditorField::Duration => match key {
                _ => input_captured = false,
            },
        }

        input_captured
    }

    pub fn take_task(&mut self) -> Task {
        std::mem::take(&mut self.task)
    }

    pub fn edit_task(&mut self, task: Task) {
        self.task = task;
        self.status = PopupStatus::InUse;
        self.task_source = TaskSource::Existing;
    }

    pub fn new_task(&mut self) {
        self.status = PopupStatus::InUse;
        self.selected_field = TaskEditorField::Description;
        self.task_source = TaskSource::New;
    }

    fn get_style(&self, field: TaskEditorField) -> Style {
        if self.selected_field == field {
            THEME.popup_selected
            //THEME.popup_focused
        } else {
            THEME.popup
        }
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
            .title(Line::from(if let TaskSource::New = self.task_source {"New Task"} else {"Edit Task"}))
            .title_bottom(format!(" [Esc] to Cancel [Enter] to Confirm "));

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
            .style(self.get_style(TaskEditorField::Cancel))
            .render(cancel_area, buf);
        Paragraph::new(" [Confirm] ")
            .style(self.get_style(TaskEditorField::Confirm))
            .render(quit_area, buf);

        let mid_horiz = Layout::horizontal([
            Constraint::Length(10),
            Constraint::Length(13),
            Constraint::Length(14),
        ]);
        let [status_area, date_area, duration_area] = mid_horiz.areas(mid_area);
        Span::styled(
            format!("Status: {}", self.task.status.get_symbol()),
            self.get_style(TaskEditorField::Status)
        )
            .render(status_area, buf);
        Span::styled(
            format!("Date: {}", self.task.date),
            self.get_style(TaskEditorField::Date)
        )
            .render(date_area, buf);
        Span::styled(
            format!("Dur: {}", self.task.duration),
            self.get_style(TaskEditorField::Duration)
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
            .style(self.get_style(TaskEditorField::Description))
            .render(text_area, buf);

    }

}
