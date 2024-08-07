use ratatui::{prelude::*, widgets::*};
use crossterm::event::{KeyCode};
use crate::{
    theme::{THEME},
    popup::{PopupStatus, NewTaskPopup},
};

#[derive(Default)]
pub enum TaskStatus {
    #[default]
    NotStarted,
    InProgress,
    Finished,
}

#[derive(Default)]
pub struct Task {
    pub name: String,
    pub status: TaskStatus,
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

    pub new_task_window: NewTaskPopup,
}

impl TaskListTab {
    pub fn handle_input(&mut self, key: KeyCode) {
        if PopupStatus::InUse == self.new_task_window.status {
            self.new_task_window.handle_input(key);
            if PopupStatus::Confirmed == self.new_task_window.status {
                self.task_lists[self.selected].tasks.push(self.new_task_window.take_task());
                self.new_task_window.status = PopupStatus::Closed;
            }
        } else {
            match key {
                KeyCode::Char('h') => self.previous_tab(),
                KeyCode::Char('l') => self.next_tab(),
                KeyCode::Char('k') => self.task_lists[self.selected].previous_task(),
                KeyCode::Char('j') => self.task_lists[self.selected].next_task(),
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

        let index = self.task_lists[self.selected].selected;
        self.task_lists[self.selected].tasks.remove(index);
        if index == self.task_lists[self.selected].tasks.len() {
            self.task_lists[self.selected].previous_task();
        }
    }

    fn interact(&mut self) {
        if self.task_lists.len() == 0 { return; }
        if self.task_lists[self.selected].tasks.len() == 0 { return; }

        let index = self.task_lists[self.selected].selected;
        self.task_lists[self.selected].tasks[index].status = match self.task_lists[self.selected].tasks[index].status {
            TaskStatus::NotStarted => TaskStatus::InProgress,
            TaskStatus::InProgress => TaskStatus::Finished,
            TaskStatus::Finished => TaskStatus::NotStarted,
        }
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
