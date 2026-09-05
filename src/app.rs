use std::{collections::HashSet, io, time::Duration};

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use futures_util::StreamExt;
use ratatui::{DefaultTerminal, layout::Rect};
use tokio::time::{self, MissedTickBehavior};

use crate::{component::Component, ui};

pub struct App {
    components: Vec<Box<dyn Component>>,
    active: usize,
    running: bool,
    tab_areas: Vec<Rect>,
}

impl App {
    pub fn new(components: Vec<Box<dyn Component>>) -> io::Result<Self> {
        if components.is_empty() {
            return Err(io::Error::other("Register at least one component"));
        }
        let mut ids = HashSet::new();
        if components.iter().any(|component| !ids.insert(component.id())) {
            return Err(io::Error::other("Component IDs must be unique"));
        }
        let active = components.iter().position(|component| component.id() == "ping").unwrap_or(0);
        Ok(Self { components, active, running: true, tab_areas: Vec::new() })
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let result = self.run_loop(terminal).await;
        for component in &mut self.components {
            component.shutdown().await;
        }
        result
    }

    async fn run_loop(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut events = EventStream::new();
        let mut tick = time::interval(Duration::from_millis(100));
        tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

        while self.running {
            for component in &mut self.components {
                component.update();
            }
            terminal.draw(|frame| {
                self.tab_areas = ui::render(frame, &mut self.components, self.active);
            })?;
            tokio::select! {
                _ = tick.tick() => {}
                event = events.next() => match event {
                    Some(Ok(event)) => self.handle_event(&event),
                    Some(Err(error)) => return Err(error),
                    None => break,
                }
            }
        }
        Ok(())
    }

    fn activate(&mut self, index: usize) {
        if index < self.components.len() {
            for component in &mut self.components {
                component.reset_layout();
            }
            self.active = index;
        }
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Release {
                return;
            }
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('c')
            {
                self.running = false;
                return;
            }
        }
        if matches!(event, Event::Resize(..)) {
            self.tab_areas.clear();
            for component in &mut self.components {
                component.reset_layout();
            }
        }
        if let Event::Mouse(mouse) = event
            && mouse.kind == MouseEventKind::Down(MouseButton::Left)
            && let Some(index) = self.tab_areas.iter().position(|area| {
                area.contains((mouse.column, mouse.row).into())
            })
        {
            self.activate(index);
            return;
        }
        // Text fields get first refusal: q and digits remain text while editing.
        if self.components[self.active].handle_event(event) {
            return;
        }
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.running = false,
                KeyCode::Tab => self.activate((self.active + 1) % self.components.len()),
                KeyCode::BackTab => self.activate((self.active + self.components.len() - 1) % self.components.len()),
                KeyCode::Char(digit @ '1'..='9') => self.activate((digit as u8 - b'1') as usize),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests;
