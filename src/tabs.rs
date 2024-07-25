use ratatui::{prelude::*, widgets::*};

pub struct MainTab {

}

impl Widget for &MainTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("main tab test").render(area, buf);
    }
}

pub struct CalenderTab {

}

impl Widget for &CalenderTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("cal tab test").render(area, buf);
    }
}

pub struct OptionsTab {
}

impl Widget for &OptionsTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("options tab test").render(area, buf);
    }
}

pub struct ProfileTab {
}

impl Widget for &ProfileTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("options tab test").render(area, buf);
    }
}
