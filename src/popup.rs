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

#[derive(Default)]
pub enum PopupStatus {
    InUse,
    Finished,
    #[default]
    Closed,
}

#[derive(Default)]
pub struct NewTaskPopup {
    pub status: PopupStatus,
    pub text: String,
}

impl NewTaskPopup {
    pub fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => self.text.push(c),
            KeyCode::Backspace => { self.text.pop(); },
            KeyCode::Enter => self.status = PopupStatus::Finished,
            _ => {},
        }
    }
}

impl Widget for &NewTaskPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([5]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let horizontal = Layout::horizontal([60]).flex(Flex::Center);
        let [area] = horizontal.areas(area);

        let window = Block::bordered()
            .border_style(THEME.popup)
            .title(Line::from("New Task"));

        Paragraph::new(self.text.as_str())
            .block(window)
            .wrap(Wrap {trim: true})
            .render(area, buf);
    }

}
