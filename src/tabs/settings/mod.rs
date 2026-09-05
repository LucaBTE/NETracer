mod ui;

use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::{Frame, layout::Rect, widgets::TableState};

use crate::{
    components::Component,
    theme::{self, ThemeId},
};

pub struct SettingsTab {
    selected: usize,
    table_state: TableState,
    table_area: Rect,
}

impl SettingsTab {
    pub fn new() -> Self {
        let selected = ThemeId::ALL
            .iter()
            .position(|theme| *theme == theme::active())
            .unwrap_or(0);
        let mut table_state = TableState::default();
        table_state.select(Some(selected));
        Self {
            selected,
            table_state,
            table_area: Rect::default(),
        }
    }

    fn select(&mut self, index: usize) {
        self.selected = index.min(ThemeId::ALL.len() - 1);
        self.table_state.select(Some(self.selected));
        theme::select(ThemeId::ALL[self.selected]);
    }

    fn move_selection(&mut self, direction: isize) {
        self.select(
            self.selected
                .saturating_add_signed(direction)
                .min(ThemeId::ALL.len() - 1),
        );
    }
}

impl Default for SettingsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SettingsTab {
    fn id(&self) -> &'static str {
        "settings"
    }
    fn title(&self) -> &'static str {
        "Settings"
    }
    fn help(&self) -> &'static str {
        "↑/↓: choose theme  CLICK: choose  Q: quit"
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => match key.code {
                KeyCode::Up => {
                    self.move_selection(-1);
                    true
                }
                KeyCode::Down => {
                    self.move_selection(1);
                    true
                }
                _ => false,
            },
            Event::Mouse(mouse)
                if mouse.kind == MouseEventKind::Down(MouseButton::Left)
                    && self.table_area.contains((mouse.column, mouse.row).into()) =>
            {
                let first_row = self.table_area.y + 2;
                if mouse.row >= first_row {
                    let index = self.table_state.offset() + usize::from(mouse.row - first_row);
                    if index < ThemeId::ALL.len() {
                        self.select(index);
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        ui::render(frame, area, self);
    }
    fn reset_layout(&mut self) {
        self.table_area = Rect::default();
    }
}
