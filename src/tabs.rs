use ratatui::{prelude::*, widgets::*};
use crossterm::event::{KeyCode};
use crate::{THEME};

pub struct Task {
    pub name: String,
}

pub struct TaskList {
    pub name: String,
    pub tasks: Vec<Task>,
}

pub struct TaskListTab {
    pub selected_list_index: usize,
    pub task_lists: Vec<TaskList>,
}

impl TaskListTab {
    pub fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('h') => self.previous_tab(),
            KeyCode::Char('l') => self.next_tab(),
            _ => {},
        }
    }

    fn next_tab(&mut self) {
        if self.selected_list_index + 1 == self.task_lists.len() {
            self.selected_list_index = 0;
        } else {
            self.selected_list_index += 1;
        }
    }

    fn previous_tab(&mut self) {
        if self.selected_list_index == 0 {
            self.selected_list_index = if self.task_lists.len() == 0 {
                0
            } else {
                self.task_lists.len() - 1
            };
        } else {
            self.selected_list_index -= 1;
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
                .style(if i == self.selected_list_index {THEME.root_tab_selected} else {THEME.root}));
        }
        Line::from(spans).style(THEME.root).render(task_bar, buf);

        // Task List Rendering
        let mut lines: Vec<Line> = Vec::with_capacity(self.task_lists[self.selected_list_index].tasks.len());
        for i in 0..self.task_lists[self.selected_list_index].tasks.len() {
            lines.push(Line::from(vec![
                Span::from(" > "),
                Span::from(format!(" {} ", self.task_lists[self.selected_list_index].tasks[i].name)),
            ]));
        }
        Text::from(lines).render(tasks_area, buf);
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
