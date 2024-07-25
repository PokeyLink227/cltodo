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
use crate::{
    tabs::*,
    theme::{THEME},
};

mod tui;
mod theme;
mod tabs;

enum RunningMode {
    Running,
    Exiting,
}

enum Tab {
    Main,
    Calender,
    Options,
    Profile,
}

pub struct App {
    mode: RunningMode,
    current_tab: Tab,

    main_tab: MainTab,
    calender_tab: CalenderTab,
    options_tab: OptionsTab,
    profile_tab: ProfileTab,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ]);
        let [title_bar, canvas, bottom_bar] = vertical.areas(area);

        //Block::new().style(THEME.root).render(area, buf);

        self.render_title_bar(title_bar, buf);
        match self.current_tab {
            Tab::Main => self.main_tab.render(canvas, buf),
            Tab::Calender => self.calender_tab.render(canvas, buf),
            Tab::Options => self.options_tab.render(canvas, buf),
            Tab::Profile => self.profile_tab.render(canvas, buf),
        }
        self.render_bottom_bar(bottom_bar, buf);
    }

}

impl App {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while let RunningMode::Running = self.mode {
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
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.mode = RunningMode::Exiting,
                        KeyCode::Tab => self.next_tab(),
                        _ => {},
                    }
                }
            }
        }
        Ok(())
    }

    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Main => Tab::Calender,
            Tab::Calender => Tab::Options,
            Tab::Options => Tab::Profile,
            Tab::Profile => Tab::Main,
        }
    }

    fn render_title_bar(&self, area: Rect, buf: &mut Buffer) {
        let horizontal = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(7),
            Constraint::Length(10),
            Constraint::Length(9),
            Constraint::Length(9),
        ]);
        let [app_name, list_tab, calender_tab, options_tab, profile_tab] = horizontal.areas(area);

        Block::new().style(THEME.root).render(area, buf);
        Paragraph::new("CL-TODO").render(app_name, buf);
        Paragraph::new(" Lists ").style(if let Tab::Main = self.current_tab {THEME.root_tab_selected} else {THEME.root}).render(list_tab, buf);
        Paragraph::new(" Calender ").style(if let Tab::Calender = self.current_tab {THEME.root_tab_selected} else {THEME.root}).render(calender_tab, buf);
        Paragraph::new(" Options ").style(if let Tab::Options = self.current_tab {THEME.root_tab_selected} else {THEME.root}).render(options_tab, buf);
        Paragraph::new(" Profile ").style(if let Tab::Profile = self.current_tab {THEME.root_tab_selected} else {THEME.root}).render(profile_tab, buf);
    }

    fn render_bottom_bar(&self, area: Rect, buf: &mut Buffer) {
        let keys = [
            ("Q/Esc", "Quit"),
            ("Tab", "Switch Tab"),
            ("J", "Down"),
            ("K", "Up"),
        ];

        let spans: Vec<Span> = keys
            .iter()
            .flat_map(|(key, desc)| {
                let key = Span::from(format!(" {key} ")).style(THEME.key_bind);
                let desc = Span::from(format!(" {desc} ")).style(THEME.key_desc);
                [key, desc]
            })
            .collect();

        Line::from(spans).centered().render(area, buf);
    }

}

fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let mut app = App {mode: RunningMode::Running, current_tab: Tab::Main, main_tab: MainTab {}, calender_tab: CalenderTab {}, options_tab: OptionsTab {}, profile_tab: ProfileTab {}};
    app.run(&mut terminal)?;
    tui::restore()
}
