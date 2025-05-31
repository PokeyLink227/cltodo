use crate::{
    tabs::{Task, TaskStatus, disp_md},
    theme::THEME,
    widgets::TextEntry,
};
use chrono::{Datelike, Days, NaiveDate};
use crossterm::event::KeyCode;
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Block, BorderType, Clear, Paragraph, Wrap},
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
    //Cancel,
    //Confirm,
}

impl TaskEditorField {
    fn next(&mut self) {
        *self = match self {
            TaskEditorField::Description => TaskEditorField::Status,
            TaskEditorField::Status => TaskEditorField::Date,
            TaskEditorField::Date => TaskEditorField::Duration,
            TaskEditorField::Duration => TaskEditorField::Description,
            //TaskEditorField::Cancel => TaskEditorField::Confirm,
            //TaskEditorField::Confirm => TaskEditorField::Description,
        }
    }

    fn previous(&mut self) {
        *self = match self {
            TaskEditorField::Description => TaskEditorField::Duration,
            TaskEditorField::Status => TaskEditorField::Description,
            TaskEditorField::Date => TaskEditorField::Status,
            TaskEditorField::Duration => TaskEditorField::Date,
            //TaskEditorField::Cancel => TaskEditorField::Duration,
            //TaskEditorField::Confirm => TaskEditorField::Cancel,
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

    desc_field: TextEntry,

    editing_date: bool,
    date_field: TextEntry,
}

impl TaskEditorPopup {
    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        match self.selected_field {
            TaskEditorField::Description => match key {
                KeyCode::Char(c) => self.desc_field.insert(c),
                KeyCode::Backspace => self.desc_field.remove(),
                KeyCode::Left => self.desc_field.move_cursor_left(),
                KeyCode::Right => self.desc_field.move_cursor_right(),
                _ => {}
            },
            /*
            TaskEditorField::Cancel => match key {
                KeyCode::Char('e') => self.status = PopupStatus::Canceled,
                _ => input_captured = false,
            },
            TaskEditorField::Confirm => match key {
                KeyCode::Char('e') => self.status = PopupStatus::Confirmed,
                _ => input_captured = false,
            },
            */
            TaskEditorField::Status => match key {
                KeyCode::Char('1') => self.task.status = TaskStatus::NotStarted,
                KeyCode::Char('2') => self.task.status = TaskStatus::InProgress,
                KeyCode::Char('3') => self.task.status = TaskStatus::Finished,
                KeyCode::Char('j') => self.task.status.cycle_next(),
                _ => {}
            },
            TaskEditorField::Date => match key {
                KeyCode::Char('j') => self.task.date = self.task.date.succ_opt().unwrap(),
                KeyCode::Char('k') => self.task.date = self.task.date.pred_opt().unwrap(),
                KeyCode::Tab | KeyCode::Char('/') if self.editing_date => {
                    self.submit_date();
                }
                KeyCode::Backspace => {
                    if self.editing_date {
                        self.date_field.remove();
                    }
                }
                KeyCode::Char(c) if c >= '0' && c <= '9' || c == '+' || c == '-' => {
                    self.editing_date = true;
                    self.date_field.insert(c);
                }
                _ => {}
            },
            TaskEditorField::Duration => match key {
                _ => {}
            },
        }

        match key {
            KeyCode::Enter => {
                if self.editing_date {
                    self.submit_date();
                } else {
                    self.status = PopupStatus::Confirmed;
                    self.task.name = self.desc_field.take();
                }
            }
            KeyCode::Esc => self.status = PopupStatus::Closed,
            KeyCode::Tab => self.selected_field.next(),
            KeyCode::BackTab => self.selected_field.previous(),
            _ => {}
        }

        // ensure app cannot be exited through hotkey while popup is open
        true
    }

    pub fn take_task(&mut self) -> Task {
        std::mem::take(&mut self.task)
    }

    pub fn edit_task(&mut self, task: Task) {
        self.task = task;
        self.desc_field.set_text(self.task.name.clone());
        self.desc_field.move_cursor_end();
        self.status = PopupStatus::InUse;
        self.task_source = TaskSource::Existing;
    }

    pub fn new_task(&mut self) {
        self.status = PopupStatus::InUse;
        self.selected_field = TaskEditorField::Description;
        self.task_source = TaskSource::New;

        self.task.date = chrono::offset::Local::now().date_naive();
        self.desc_field.move_cursor_end();
    }

    fn get_style(&self, field: TaskEditorField) -> Style {
        if self.selected_field == field {
            THEME.popup_selected
            //THEME.popup_focused
        } else {
            THEME.popup
        }
    }

    // consumes date entry string and updates date if pattern is valid
    fn submit_date(&mut self) {
        match self.parse_date() {
            None => {}
            Some(d) => self.task.date = d,
        }
        self.editing_date = false;
        self.date_field.clear();
    }

    fn parse_date(&mut self) -> Option<NaiveDate> {
        let today = chrono::offset::Local::now().date_naive();

        // +d adds d days to todays date
        if self.date_field.get_str().chars().nth(0)? == '+' {
            today.checked_add_days(Days::new(
                self.date_field.get_str().get(1..)?.parse::<u64>().ok()?,
            ))
        // -d adds d days to todays date
        } else if self.date_field.get_str().chars().nth(0)? == '-' {
            today.checked_sub_days(Days::new(
                self.date_field.get_str().get(1..)?.parse::<u64>().ok()?,
            ))
        // mmdd sets date to (current_year, mm, dd)
        } else if self.date_field.get_str().len() == 4 {
            NaiveDate::from_ymd_opt(
                today.year(),
                self.date_field.get_str().get(0..2)?.parse::<u32>().ok()?,
                self.date_field.get_str().get(2..4)?.parse::<u32>().ok()?,
            )
        // yyyymmdd sets date to (yyyy, mm, dd)
        } else if self.date_field.get_str().len() == 8 {
            NaiveDate::from_ymd_opt(
                self.date_field.get_str().get(0..4)?.parse::<i32>().ok()?,
                self.date_field.get_str().get(4..6)?.parse::<u32>().ok()?,
                self.date_field.get_str().get(6..8)?.parse::<u32>().ok()?,
            )
        } else {
            None
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
            .border_type(BorderType::Rounded)
            .title(Line::from(if let TaskSource::New = self.task_source {
                "New Task"
            } else {
                "Edit Task"
            }))
            .title_bottom(
                Line::raw(format!(" [Esc] to Cancel [Enter] to Confirm ")).right_aligned(),
            );

        let win_area = window.inner(area);
        Clear.render(win_area, buf);
        window.render(area, buf);

        let vert = Layout::vertical([1, 1, 1]);
        let [top_area, mid_area, _bot_area] = vert.areas(win_area);

        let mid_horiz = Layout::horizontal([
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(14),
        ]);
        let [status_area, date_area, duration_area] = mid_horiz.areas(mid_area);
        Span::styled(
            format!("Status: {}", self.task.status.get_symbol()),
            self.get_style(TaskEditorField::Status),
        )
        .render(status_area, buf);
        Span::styled(
            format!(
                "Date: {}",
                if self.editing_date {
                    self.date_field.get_str().to_owned()
                } else {
                    disp_md(self.task.date)
                }
            ),
            self.get_style(TaskEditorField::Date),
        )
        .render(date_area, buf);
        if self.selected_field == TaskEditorField::Date && self.editing_date {
            buf[(
                date_area.x + 6 + self.date_field.get_cursor_pos() as u16,
                date_area.y,
            )]
                .set_style(THEME.popup_cursor);
        }

        Span::styled(
            format!("Dur: {}", self.task.duration),
            self.get_style(TaskEditorField::Duration),
        )
        .render(duration_area, buf);

        //let text_entry = Paragraph::new(self.text.as_str()).wrap(Wrap {trim: true});
        let top_horiz = Layout::horizontal([Constraint::Min(0)]);
        let [text_area] = top_horiz.areas(top_area);
        Line::from(vec![
            Span::from("Desc: "),
            Span::from(self.desc_field.get_str()),
        ])
        .style(self.get_style(TaskEditorField::Description))
        .render(text_area, buf);

        if self.selected_field == TaskEditorField::Description {
            buf[(
                text_area.x + 6 + self.desc_field.get_cursor_pos() as u16,
                text_area.y,
            )]
                .set_style(THEME.popup_cursor);
        }
    }
}

#[derive(Default)]
pub struct TextEntryPopup {
    pub text_field: TextEntry,
    pub title: String,
    pub status: PopupStatus,
    pub max_lines: u16,
}

impl TextEntryPopup {
    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Enter => self.confirm(),
            KeyCode::Esc => self.cancel(),
            KeyCode::Char(c) => self.text_field.insert(c),
            KeyCode::Backspace => self.text_field.remove(),
            KeyCode::Left => self.text_field.move_cursor_left(),
            KeyCode::Right => self.text_field.move_cursor_right(),
            _ => {}
        }

        true
    }

    pub fn new(title: String, max_lines: u16) -> Self {
        TextEntryPopup {
            text_field: TextEntry::new(),
            title,
            status: PopupStatus::Closed,
            max_lines,
        }
    }

    fn confirm(&mut self) {
        self.status = PopupStatus::Confirmed;
    }

    fn cancel(&mut self) {
        self.status = PopupStatus::Canceled;
    }

    pub fn close(&mut self) {
        self.status = PopupStatus::Closed;
    }

    pub fn show(&mut self) {
        self.status = PopupStatus::InUse;
    }

    pub fn reset(&mut self) {
        self.close();
        self.text_field.clear();
    }

    pub fn take(&mut self) -> String {
        std::mem::take(&mut self.text_field.take())
    }
}

impl Widget for &TextEntryPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([self.max_lines + 2]).flex(Flex::Center);
        let horizontal = Layout::horizontal([60]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);

        let window = Block::bordered()
            .style(THEME.popup)
            .border_style(THEME.popup)
            .border_type(BorderType::Rounded)
            .title_top(self.title.as_str())
            .title_bottom(Line::raw(" [Esc] to Cancel [Enter] to Confirm ").right_aligned());

        let win_area = window.inner(area);
        Clear.render(win_area, buf);
        window.render(area, buf);

        Paragraph::new(self.text_field.get_str())
            .wrap(Wrap { trim: true })
            .style(THEME.popup_focused)
            .render(win_area, buf);

        let cursor_pos = self.text_field.get_cursor_pos() as u16;
        buf[(win_area.x + cursor_pos % 58, win_area.y + cursor_pos / 58)]
            .set_style(THEME.popup_cursor);
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum ConfirmationField {
    Yes,
    #[default]
    No,
}

impl ConfirmationField {
    pub fn cycle_next(&mut self) {
        *self = match self {
            ConfirmationField::No => ConfirmationField::Yes,
            ConfirmationField::Yes => ConfirmationField::No,
        }
    }
}

pub struct ConfirmationPopup {
    pub title: String,
    pub body: String,
    pub status: PopupStatus,

    selected_field: ConfirmationField,
}

impl ConfirmationPopup {
    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Tab => {
                self.selected_field.cycle_next();
            }
            KeyCode::BackTab => {
                self.selected_field.cycle_next();
            }
            KeyCode::Char('y') => {
                self.selected_field = ConfirmationField::Yes;
                self.status = PopupStatus::Confirmed;
            }
            KeyCode::Char('n') => {
                self.selected_field = ConfirmationField::No;
                self.status = PopupStatus::Confirmed;
            }
            KeyCode::Esc => {
                self.selected_field = ConfirmationField::No;
                self.status = PopupStatus::Canceled;
            }
            KeyCode::Enter => {
                self.status = PopupStatus::Confirmed;
            }
            _ => {}
        }

        true
    }

    pub fn new(new_title: String, new_body: String) -> ConfirmationPopup {
        ConfirmationPopup {
            selected_field: ConfirmationField::No,
            title: new_title,
            body: new_body,
            status: PopupStatus::Closed,
        }
    }

    pub fn show(&mut self) {
        self.selected_field = ConfirmationField::No;
        self.status = PopupStatus::InUse;
    }

    pub fn close(&mut self) {
        self.status = PopupStatus::Closed;
    }

    pub fn decision(&self) -> bool {
        match self.selected_field {
            ConfirmationField::No => false,
            ConfirmationField::Yes => true,
        }
    }
}

impl Widget for &ConfirmationPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([5]).flex(Flex::Center);
        let horizontal = Layout::horizontal([45]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);

        let window = Block::bordered()
            .style(THEME.popup)
            .border_style(THEME.popup)
            .border_type(BorderType::Rounded)
            .title(Span::from(&self.title));
        /*
        .title(
            Title::from(format!(" [Esc] to Cancel [Enter] to Confirm "))
                .alignment(Alignment::Right)
                .position(Position::Bottom),
        );
        */

        let win_area = window.inner(area);
        Clear.render(win_area, buf);
        window.render(area, buf);

        let vertical = Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ]);
        let [body_area, _gap, button_area] = vertical.areas(win_area);

        Paragraph::new(self.body.as_str())
            .style(THEME.popup)
            .alignment(Alignment::Center)
            .render(body_area, buf);

        Line::from(vec![
            Span::from("[No]").style(if self.selected_field == ConfirmationField::No {
                THEME.popup_selected
            } else {
                THEME.popup
            }),
            Span::from("               "),
            Span::from("[Yes]").style(if self.selected_field == ConfirmationField::Yes {
                THEME.popup_selected
            } else {
                THEME.popup
            }),
        ])
        .centered()
        .render(button_area, buf);
    }
}
