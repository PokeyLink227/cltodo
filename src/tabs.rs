use ratatui::{prelude::*, widgets::*};
use crossterm::event::{KeyCode};
use crate::{THEME};

pub enum TaskStatus {
    NotStarted,
    InProgress,
    Finished,
}

pub struct Task {
    pub name: String,
    pub status: TaskStatus,
}

pub struct TaskList {
    pub name: String,
    pub selected: usize,
    pub tasks: Vec<Task>,
}

pub struct TaskListTab {
    pub selected: usize,
    pub task_lists: Vec<TaskList>,
}

impl TaskListTab {
    pub fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('h') => self.previous_tab(),
            KeyCode::Char('l') => self.next_tab(),
            KeyCode::Char('k') => self.previous_task(),
            KeyCode::Char('j') => self.next_task(),
            _ => {},
        }
    }

    fn next_tab(&mut self) {
        if self.selected + 1 == self.task_lists.len() {
            self.selected = 0;
        } else {
            self.selected += 1;
        }
    }

    fn previous_tab(&mut self) {
        if self.selected == 0 {
            self.selected = if self.task_lists.len() == 0 {
                0
            } else {
                self.task_lists.len() - 1
            };
        } else {
            self.selected -= 1;
        }
    }

    fn next_task(&mut self) {
        if self.task_lists.len() == 0 { return; }

        if self.task_lists[self.selected].selected + 1 == self.task_lists[self.selected].tasks.len() {
            self.task_lists[self.selected].selected = 0;
        } else {
            self.task_lists[self.selected].selected += 1;
        }
    }

    fn previous_task(&mut self) {
        if self.task_lists.len() == 0 { return; }

        if self.task_lists[self.selected].selected == 0 {
            self.task_lists[self.selected].selected = if self.task_lists[self.selected].tasks.len() == 0 {
                0
            } else {
                self.task_lists[self.selected].tasks.len() - 1
            }
        } else {
            self.task_lists[self.selected].selected -= 1;
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
        spans.push(Span::from("Task Lists:"));
        for i in 0..self.task_lists.len() {
            spans.push(Span::from(format!(" {} ", self.task_lists[i].name))
                .style(if i == self.selected {THEME.root_tab_selected} else {THEME.root}));
        }
        Line::from(spans).style(THEME.root).render(task_bar, buf);

        // Task List Rendering
        let mut lines: Vec<Line> = Vec::with_capacity(self.task_lists[self.selected].tasks.len());
        for i in 0..self.task_lists[self.selected].tasks.len() {
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} ", if i == self.task_lists[self.selected].selected {'>'} else {' '}),
                    if i == self.task_lists[self.selected].selected {THEME.task_selected} else {THEME.task}
                ),
                Span::styled(
                    format!(
                        "[{}] {} ",
                        match self.task_lists[self.selected].tasks[i].status {
                            TaskStatus::NotStarted => ' ',
                            TaskStatus::InProgress => '-',
                            TaskStatus::Finished => 'X',
                        },
                        self.task_lists[self.selected].tasks[i].name
                    ),
                    if i == self.task_lists[self.selected].selected {THEME.task_selected} else {THEME.task}
                ),
            ]));
        }
        let tasks_border = Block::bordered().border_style(THEME.task_border).border_type(BorderType::Thick);
        Text::from(lines).render(tasks_border.inner(tasks_area), buf);
        tasks_border.render(tasks_area, buf);
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
