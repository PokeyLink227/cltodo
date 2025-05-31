use crate::theme::THEME;
use chrono::{Datelike, Weekday};
use ratatui::{layout::Offset, prelude::*};

pub struct TextEntry {
    text: String,
    cursor_pos: usize,
    max_len: usize,
}

impl Default for TextEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl TextEntry {
    pub fn new() -> Self {
        TextEntry {
            text: String::new(),
            cursor_pos: 0,
            max_len: usize::MAX,
        }
    }

    pub fn with_text(text: String) -> Self {
        let cursor_pos = text.len();
        TextEntry {
            text,
            cursor_pos,
            max_len: usize::MAX,
        }
    }

    pub fn with_max(mut self, len: usize) -> Self {
        self.max_len = len;
        self
    }

    pub fn take(&mut self) -> String {
        self.move_cursor_home();
        std::mem::take(&mut self.text)
    }

    pub fn set_text(&mut self, new_text: String) {
        self.text = new_text;
        self.cursor_pos = self.text.len();
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.move_cursor_home();
    }

    pub fn get_str(&self) -> &str {
        self.text.as_str()
    }

    pub fn get_cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    fn byte_index(&self) -> usize {
        self.text
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_pos)
            .unwrap_or(self.text.len())
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
        }
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_pos = self.text.len();
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.text.len() {
            self.cursor_pos += 1;
        }
    }

    pub fn insert(&mut self, c: char) {
        if self.cursor_pos < self.max_len {
            self.text.insert(self.byte_index(), c);
            self.move_cursor_right();
        }
    }

    pub fn remove(&mut self) {
        if self.text.is_empty() {
            return;
        }

        // stops backspace from acting like del when at the beginning of the string
        if self.cursor_pos == 0 {
            return;
        }

        self.move_cursor_left();
        self.text.remove(self.byte_index());
    }
}

static DAYS: [&'static str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

#[derive(Default)]
pub struct Calendar {}

impl Calendar {
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        Line::from(
            DAYS.iter()
                .map(|day| Span::from(format!("{} ", day)))
                .collect::<Vec<Span>>(),
        )
        .render(area, buf);

        let date = chrono::offset::Local::now().date_naive();
        let first = date.with_day(1).unwrap();
        let cal_start = first.week(Weekday::Sun).first_day();
        let lines: Vec<Line> = cal_start
            .iter_weeks()
            .take(6)
            .map(|week| {
                Line::from(
                    week.iter_days()
                        .take(7)
                        .map(|day| {
                            Span::styled(
                                format!("{:2} ", day.day()),
                                if day.month() == date.month() {
                                    if day == date {
                                        THEME.calendar.today
                                    } else {
                                        THEME.calendar.this_month
                                    }
                                } else {
                                    THEME.calendar.other_month
                                },
                            )
                        })
                        .collect::<Vec<Span>>(),
                )
            })
            .collect();

        Text::from(lines).render(area.offset(Offset { x: 0, y: 1 }), buf);
    }
}
