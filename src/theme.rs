use ratatui::style::{Color, Style};

pub struct Theme {
    pub root: Style,
    pub root_tab_selected: Style,
    pub root_cursor: Style,
    pub command_error: Style,
    pub popup: Style,
    pub popup_focused: Style,
    pub popup_selected: Style,
    pub popup_cursor: Style,
    pub task: Style,
    pub task_selected: Style,
    pub task_list: Style,
    pub task_list_selected: Style,
    pub task_border: Style,
    pub task_title: Style,
    pub key_bind: Style,
    pub key_desc: Style,
    pub calendar: CalendarStyle,
}

pub struct CalendarStyle {
    pub today: Style,
    pub this_month: Style,
    pub other_month: Style,
}

pub const THEME: Theme = Theme {
    root: Style::new().fg(WHITE).bg(DARKER_GRAY),
    root_cursor: Style::new().fg(DARKER_GRAY).bg(WHITE),
    root_tab_selected: Style::new().fg(YELLOW).bg(DARK_GRAY),
    command_error: Style::new().fg(WHITE).bg(RED),
    popup: Style::new().fg(WHITE),
    popup_focused: Style::new().fg(WHITE).bg(DARK_GRAY),
    popup_selected: Style::new().fg(YELLOW).bg(DARK_GRAY),
    popup_cursor: Style::new().fg(DARK_GRAY).bg(YELLOW),
    task: Style::new().fg(WHITE),
    task_selected: Style::new().fg(YELLOW),
    task_list: Style::new().fg(WHITE),
    task_list_selected: Style::new().fg(DARKER_GRAY).bg(YELLOW),
    task_border: Style::new().fg(DARK_GRAY),
    task_title: Style::new().fg(GRAY),
    key_bind: Style::new().fg(BLACK).bg(DARK_GRAY),
    key_desc: Style::new().fg(DARK_GRAY).bg(BLACK),
    calendar: CalendarStyle {
        today: Style::new().fg(BLUE_4),
        this_month: Style::new().fg(WHITE),
        other_month: Style::new().fg(GRAY),
    },
};

const WHITE: Color = Color::Rgb(238, 238, 238);
const LIGHT_GRAY: Color = Color::Rgb(188, 188, 188);
const GRAY: Color = Color::Rgb(128, 128, 128);
const DARK_GRAY: Color = Color::Rgb(68, 68, 68);
const DARKER_GRAY: Color = Color::Rgb(38, 38, 38);
const BLACK: Color = Color::Rgb(8, 8, 8);
const YELLOW: Color = Color::Rgb(240, 180, 30);
const RED: Color = Color::Rgb(210, 60, 60);

//from apollo color palette
const BLUE_4: Color = Color::from_u32(0x0073bed3);
