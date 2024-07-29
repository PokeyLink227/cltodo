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
        let horizontal = Layout::horizontal([60]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);

        let window = Block::bordered()
            .border_style(THEME.popup)
            .title(Line::from("New Task"));

        let win_area = window.inner(area);
        let vert = Layout::vertical([1, 1, 1]);
        let [text_area, mid, bot] = vert.areas(win_area);

        //let text_entry = Paragraph::new(self.text.as_str()).wrap(Wrap {trim: true});

        Line::from(vec![
            Span::from("Desc: "),
            Span::from(self.text.as_str()),
        ])
            .style(THEME.popup_selected)
            .render(text_area, buf);

        window.render(area, buf);
    }

}
