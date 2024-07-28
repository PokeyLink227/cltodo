use ratatui::{
    prelude::*,
    widgets::*,
    layout::Flex,
};
use crossterm::event::{KeyCode};
use crate::theme::{THEME};

pub struct NewTaskPopup {

}

impl Widget for &NewTaskPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([5]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let horizontal = Layout::horizontal([20]).flex(Flex::Center);
        let [area] = horizontal.areas(area);

        let window = Block::bordered()
            .border_style(THEME.popup)
            .title(Line::from("New Task"));

        Paragraph::new("You have unsaved data")
            .block(window)
            .wrap(Wrap {trim: true})
            .render(area, buf);
    }
}
