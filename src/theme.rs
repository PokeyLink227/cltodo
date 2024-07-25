use ratatui::{Color, Modifier, Style};

pub struct Theme {
    root: Style,
    key_bind: Style,
    key_desc: Style,
}

pub const THEME: Theme = Theme {
    root: Style::default().fg(WHITE).bg(LIGHT_GRAY),
    key_bind:
}

const WHITE: Color = Color::Rgb(238, 238, 238);
const LIGHT_GRAY: Color = Color::Rgb(188, 188, 188);
const GRAY: Color = Color:Rgb(128, 128, 128);
const DARK_GRAY: Color = Color::Rgb(68, 68, 68);
