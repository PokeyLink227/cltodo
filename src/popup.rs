use crate::{
    tabs::{disp_md, Task, TaskStatus},
    theme::THEME,
    widgets::TextEntry,
};
use chrono::{Datelike, Days, NaiveDate};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Flex, Offset},
    prelude::*,
    widgets::{
        block::{Position, Title},
        Block, BorderType, Clear,
    },
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
        let mut input_captured = true;

        match self.selected_field {
            TaskEditorField::Description => match key {
                KeyCode::Char(c) => self.desc_field.insert(c),
                KeyCode::Backspace => self.desc_field.remove(),
                KeyCode::Left => self.desc_field.move_cursor_left(),
                KeyCode::Right => self.desc_field.move_cursor_right(),
                _ => input_captured = false,
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
                _ => input_captured = false,
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
                _ => input_captured = false,
            },
            TaskEditorField::Duration => match key {
                _ => input_captured = false,
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
        input_captured || key == KeyCode::Char('q')
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
            .title(
                Title::from(format!(" [Esc] to Cancel [Enter] to Confirm "))
                    .alignment(Alignment::Right)
                    .position(Position::Bottom),
            );

        let win_area = window.inner(area);
        Clear.render(win_area, buf);
        window.render(area, buf);

        let vert = Layout::vertical([1, 1, 1]);
        let [top_area, mid_area, bot_area] = vert.areas(win_area);
        /*
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
            */

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
            Span::from("█").style(THEME.popup_selected).render(
                date_area.offset(Offset {
                    x: 6 + self.date_field.get_cursor_pos() as i32,
                    y: 0,
                }),
                buf,
            );
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
            Span::from("█").style(THEME.popup_selected).render(
                text_area.offset(Offset {
                    x: 6 + self.desc_field.get_cursor_pos() as i32,
                    y: 0,
                }),
                buf,
            );
        }
    }
}
