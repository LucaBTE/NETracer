use super::*;
use crossterm::event::KeyEvent;
use ratatui::{Frame, backend::TestBackend, Terminal};
use std::{cell::Cell, rc::Rc};

struct ExtraTab { updates: Rc<Cell<u32>> }

impl Component for ExtraTab {
    fn id(&self) -> &'static str { "extra" }
    fn title(&self) -> &'static str { "Extra" }
    fn help(&self) -> &'static str { "Type q without quitting" }
    fn handle_event(&mut self, event: &Event) -> bool {
        matches!(event, Event::Key(key) if key.code == KeyCode::Char('q'))
    }
    fn update(&mut self) { self.updates.set(self.updates.get() + 1); }
    fn render(&mut self, _frame: &mut Frame, _area: Rect) {}
}

#[test]
fn accepts_extension_without_a_tab_enum_and_respects_consumed_input() {
    let updates = Rc::new(Cell::new(0));
    let mut app = App::new(vec![Box::new(ExtraTab { updates: updates.clone() })]).unwrap();
    app.handle_event(&Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)));
    assert!(app.running);
    app.handle_event(&Event::Key(KeyEvent::new(KeyCode::Char('9'), KeyModifiers::NONE)));
    assert_eq!(app.active, 0);
    app.components[0].update();
    assert_eq!(updates.get(), 1);
    app.handle_event(&Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
    assert!(!app.running);
}

#[test]
fn rejects_empty_and_duplicate_registration() {
    assert!(App::new(Vec::new()).is_err());
    let make = || Box::new(ExtraTab { updates: Rc::new(Cell::new(0)) }) as Box<dyn Component>;
    assert!(App::new(vec![make(), make()]).is_err());
}

#[test]
fn tiny_terminal_has_no_mouse_targets() {
    let component = Box::new(ExtraTab { updates: Rc::new(Cell::new(0)) });
    let mut app = App::new(vec![component]).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(20, 5)).unwrap();
    terminal.draw(|frame| {
        app.tab_areas = ui::render(frame, &mut app.components, 0);
    }).unwrap();
    assert!(app.tab_areas.is_empty());
}
