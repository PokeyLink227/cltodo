use chrono::{Datelike, Weekday};
use ratatui::{
    prelude::*,
    widgets::*,
    layout::Offset,
};
//use crossterm::event::{KeyCode};
use crate::{
    theme::{THEME},
};
use itertools::Itertools;

#[derive(Default)]
pub struct Calendar {

}

impl Calendar {

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let days = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

        let start = 1;

        Line::from(days
            .iter()
            .map(|day| {
                Span::from(format!("{} ", day))
            })
            .collect::<Vec<Span>>()
        )
            .render(area, buf);


        let date = chrono::offset::Local::now().date_naive();
        let first = date.with_day(1).unwrap();
        let cal_start = first.week(Weekday::Sun).first_day();
        let lines: Vec<Line> = cal_start
            .iter_weeks()
            .take(6)
            .map(|week| {
                Line::from(week
                    .iter_days()
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
                            }
                        )
                    })
                    .collect::<Vec<Span>>()
                )
            })
            .collect();

        Text::from(lines)
            .render(area.offset(Offset {x: 0, y: 1}), buf);

    }
}
