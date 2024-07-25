use std::io::{self, stdout};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{
        Block, Paragraph, Widget,
        block::{Position, Title},
    },
};

mod tui;

pub struct App {
    should_exit: bool,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ]);
        let [title_bar, canvas, bottom_bar] = vertical.areas(area);

        self.render_title_bar(title_bar, buf);
        self.render_bottom_bar(bottom_bar, buf);
    }

}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.size());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    self.should_exit = true;
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    fn render_title_bar(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("CL-TODO").render(area, buf);
    }

    fn render_bottom_bar(&self, area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(3),
        ]);
        let [app_name, options_bar] = horizontal.areas(area);

        Paragraph::new("asd").render(app_name, buf);
        Paragraph::new("bbb").bg(Color::Gray).render(options_bar, buf);
    }

}

fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let mut app = App {should_exit: false};
    app.run(&mut terminal);
    tui::restore()
}
