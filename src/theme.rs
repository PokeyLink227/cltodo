use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub root: Style,
    pub root_tab_selected: Style,
    pub popup: Style,
    pub popup_selected: Style,
    pub task: Style,
    pub task_selected: Style,
    pub task_border: Style,
    pub task_title: Style,
    pub key_bind: Style,
    pub key_desc: Style,
}

pub const THEME: Theme = Theme {
    root: Style::new().fg(WHITE).bg(DARKER_GRAY),
    root_tab_selected: Style::new().fg(YELLOW).bg(DARK_GRAY),
    popup: Style::new().fg(WHITE).bg(DARKER_GRAY),
    popup_selected: Style::new().fg(WHITE).bg(DARK_GRAY),
    task: Style::new(),
    task_selected: Style::new().fg(YELLOW),
    task_border: Style::new().fg(DARK_GRAY),
    task_title: Style::new().fg(GRAY),
    key_bind: Style::new().fg(BLACK).bg(DARK_GRAY),
    key_desc: Style::new().fg(DARK_GRAY).bg(BLACK),
};

const WHITE: Color = Color::Rgb(238, 238, 238);
const LIGHT_GRAY: Color = Color::Rgb(188, 188, 188);
const GRAY: Color = Color::Rgb(128, 128, 128);
const DARK_GRAY: Color = Color::Rgb(68, 68, 68);
const DARKER_GRAY: Color = Color::Rgb(38, 38, 38);
const BLACK: Color = Color::Rgb(8, 8, 8);
pub const YELLOW: Color = Color::Rgb(240, 180, 30);
