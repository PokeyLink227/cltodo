use ratatui::{
    prelude::*,
    widgets::*,
    layout::{Offset},
};
use crossterm::event::{KeyCode};
use crate::{
    theme::{THEME},
    popup::{PopupStatus, TaskEditorPopup, TaskSource},
};

#[derive(Default, Copy, Clone, Debug)]
pub struct Duration {
    pub days: u16,
    pub hours: u8,
    pub minutes: u8,
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.days, self.hours, self.minutes)
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Date {
    //pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:02}",
            match self.month {
                1  => "Jan",
                2  => "Feb",
                3  => "Mar",
                4  => "Apr",
                5  => "Ma",
                6  => "Jun",
                7  => "Jul",
                8  => "Aug",
                9  => "Sep",
                10 => "Oct",
                11 => "Nov",
                12 => "Dec",
                _  => "ERR",
            },
            self.day,
        )
    }
}

#[derive(Default, Clone)]
pub enum TaskStatus {
    #[default]
    NotStarted,
    InProgress,
    Finished,
}

#[derive(Default, Clone)]
pub struct Task {
    pub name: String,
    pub status: TaskStatus,
    pub duration: Duration,
    pub date: Date,
}

pub struct TaskList {
    pub name: String,
    pub selected: usize,
    pub tasks: Vec<Task>,
}

impl TaskList {
    fn next_task(&mut self) {
        if self.tasks.len() == 0 { return; }

        if self.selected + 1 == self.tasks.len() {
            self.selected = 0;
        } else {
            self.selected += 1;
        }
    }

    fn previous_task(&mut self) {
        if self.tasks.len() == 0 { return; }

        if self.selected == 0 {
            self.selected = self.tasks.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
}

pub struct TaskListTab {
    pub controls: [(&'static str, &'static str); 3],
    pub selected: usize,
    pub task_lists: Vec<TaskList>,

    pub new_task_window: TaskEditorPopup,
}

impl TaskListTab {
    pub fn handle_input(&mut self, key: KeyCode) {
        let selected_list = &mut self.task_lists[self.selected];

        if PopupStatus::InUse == self.new_task_window.status {
            self.new_task_window.handle_input(key);
            if PopupStatus::Confirmed == self.new_task_window.status {
                match self.new_task_window.task_source {
                    TaskSource::NewTask => selected_list.tasks.push(self.new_task_window.take_task()),
                    TaskSource::ExistingTask => selected_list.tasks[selected_list.selected] = self.new_task_window.take_task(),
                }
                self.new_task_window.status = PopupStatus::Closed;
            }
        } else {
            match key {
                KeyCode::Char('h') => self.previous_tab(),
                KeyCode::Char('l') => self.next_tab(),
                KeyCode::Char('k') => selected_list.previous_task(),
                KeyCode::Char('j') => selected_list.next_task(),
                KeyCode::Char('a') => self.new_task(),
                KeyCode::Char('e') => self.interact(),
                KeyCode::Char('d') => self.delete_task(),
                _ => {},
            }
        }
    }

    fn delete_task(&mut self) {
        if self.task_lists.len() == 0 { return; }
        if self.task_lists[self.selected].tasks.len() == 0 { return; }

        let selected_list = &mut self.task_lists[self.selected];

        selected_list.tasks.remove(index);
        if index == selected_list.tasks.len() {
            selected_list.previous_task();
        }
    }

    fn interact(&mut self) {
        if self.task_lists.len() == 0 { return; }
        if self.task_lists[self.selected].tasks.len() == 0 { return; }

        let index = self.task_lists[self.selected].selected;
        /*
        self.task_lists[self.selected].tasks[index].status = match self.task_lists[self.selected].tasks[index].status {
            TaskStatus::NotStarted => TaskStatus::InProgress,
            TaskStatus::InProgress => TaskStatus::Finished,
            TaskStatus::Finished => TaskStatus::NotStarted,
        }
        */

        self.new_task_window.edit_task(self.task_lists[self.selected].tasks[index].clone());
    }

    fn new_task(&mut self) {
        self.new_task_window.new_task();
        //self.task_lists[self.selected].tasks.push(Task {name: "test".to_string(), status: TaskStatus::NotStarted});
    }

    fn next_tab(&mut self) {
        if self.task_lists.len() == 0 { return; }

        if self.selected + 1 == self.task_lists.len() {
            self.selected = 0;
        } else {
            self.selected += 1;
        }
    }

    fn previous_tab(&mut self) {
        if self.task_lists.len() == 0 { return; }

        if self.selected == 0 {
            self.selected = self.task_lists.len() - 1;
        } else {
            self.selected -= 1;
        }
    }
}

impl Widget for &TaskListTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
        ]);
        let [task_bar, tasks_area] = vertical.areas(area);

        // Task Bar Rendering
        let mut spans: Vec<Span> = Vec::with_capacity(self.task_lists.len() + 1);
        let mut highlight_pos: Rect = Rect {x: tasks_area.x + 11, y: tasks_area.y, width: 0, height: 1};
        spans.push(Span::from("Task Lists:"));
        for i in 0..self.task_lists.len() {

            if highlight_pos.width == 0 {
                if i != self.selected {highlight_pos.x += self.task_lists[i].name.len() as u16 + 2;}
                else {highlight_pos.width = self.task_lists[i].name.len() as u16 + 2;}
            }

            spans.push(Span::from(format!(" {} ", self.task_lists[i].name))
                .style(if i == self.selected {THEME.root_tab_selected} else {THEME.root}));
        }
        Line::from(spans).style(THEME.root).render(task_bar, buf);

        // Task List Rendering
        let horizontal = Layout::horizontal([
            Constraint::Length(4),
            Constraint::Min(20),
            Constraint::Length(8),
            Constraint::Length(10),
        ]);

        let tasks_border = Block::bordered()
            .border_style(THEME.task_border)
            .border_type(BorderType::Thick);
            //.title_top(Line::from(" Date   Duration ").right_aligned());
        let mut tasks_inner_area = tasks_border.inner(tasks_area);

        tasks_border.render(tasks_area, buf);
        Block::bordered().style(THEME.task_selected).borders(Borders::TOP).border_type(BorderType::Thick).render(highlight_pos, buf);

        let [_, _, date_area, duration_area] = horizontal.areas(tasks_area);
        Span::styled("Date", THEME.task_title).render(date_area, buf);
        Span::styled("Duration", THEME.task_title).render(duration_area, buf);

        let mut index = 0;
        let selected_list = &self.task_lists[self.selected];
        for task in &selected_list.tasks {
            if !area.intersects(tasks_inner_area) { break; }

            let [mark_area, desc_area, date_area, duration_area] = horizontal.areas(tasks_inner_area);
            Span::styled(
                format!(
                    "[{}] ",
                    match selected_list.tasks[index].status {
                        TaskStatus::NotStarted => ' ',
                        TaskStatus::InProgress => '-',
                        TaskStatus::Finished => 'X',
                    }
                ),
                if index == selected_list.selected {THEME.task_selected} else {THEME.task}
            ).render(mark_area, buf);
            Span::styled(
                format!(" {} ", task.name),
                if index == selected_list.selected {THEME.task_selected} else {THEME.task}
            ).render(desc_area, buf);
            Span::from(format!(" {} ", task.date)).render(date_area, buf);
            Span::from(format!(" {} ", task.duration)).render(duration_area, buf);

            tasks_inner_area = tasks_inner_area.offset(Offset {x: 0, y: 1});
            index += 1;
        }

        // Popup Rendering
        if let PopupStatus::InUse = self.new_task_window.status {
            self.new_task_window.render(area, buf);
        }
    }
}

pub struct CalenderTab {

}

impl CalenderTab {
    pub fn handle_input(&mut self, key: KeyCode) {
    }
}

impl Widget for &CalenderTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("cal tab test").render(area, buf);
    }
}

pub struct OptionsTab {
}

impl OptionsTab {
    pub fn handle_input(&mut self, key: KeyCode) {
    }
}

impl Widget for &OptionsTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("options tab test").render(area, buf);
    }
}

pub struct ProfileTab {
}

impl ProfileTab {
    pub fn handle_input(&mut self, key: KeyCode) {
    }
}

impl Widget for &ProfileTab {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("options tab test").render(area, buf);
    }
}
