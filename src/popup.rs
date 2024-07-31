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

#[derive(Default, PartialEq)]
enum Mode {
    #[default]
    Editing,
    Navigating,
}

#[derive(Default, PartialEq)]
enum NewTaskField {
    #[default]
    Description,
    Cancel,
    Confirm,
}

// rename to TaskEditor
// create method to export/consume new task data
#[derive(Default)]
pub struct NewTaskPopup {
    pub status: PopupStatus,
    pub text: String,
    mode: Mode,
    selected_field: NewTaskField,
}

impl NewTaskPopup {
    pub fn handle_input(&mut self, key: KeyCode) {
        match self.selected_field {
            NewTaskField::Description => match self.mode {
                Mode::Editing => match key {
                    KeyCode::Char(c) => self.text.push(c),
                    KeyCode::Backspace => { self.text.pop(); },
                    KeyCode::Enter => self.status = PopupStatus::Finished,
                    _ => {},
                },
                Mode::Navigating => {},
            },
            NewTaskField::Cancel => {},
            NewTaskField::Confirm => {},
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
        let [text_area, mid, bot_area] = vert.areas(win_area);

        let bot_horiz = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(10),
            Constraint::Length(11),
        ]);
        let [_, cancel_area, quit_area] = bot_horiz.areas(bot_area);
        Paragraph::new(" [Cancel] ")
            .style(if self.selected_field == NewTaskField::Cancel {THEME.popup_selected} else {THEME.popup})
            .render(cancel_area, buf);
        Paragraph::new(" [Confirm] ").render(quit_area, buf);

        //let text_entry = Paragraph::new(self.text.as_str()).wrap(Wrap {trim: true});

        Line::from(vec![
            Span::from("Desc: "),
            Span::from(self.text.as_str()),
        ])
            .style(if self.selected_field == NewTaskField::Description {THEME.popup_selected} else {THEME.popup})
            .render(text_area, buf);

        window.render(area, buf);
    }

}
