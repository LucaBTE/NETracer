use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::ui;

pub struct App {
    running: bool,
}

//App functions
impl App {
    //Initialize app state
    pub fn new() -> Self {
        Self { running: true }
    }

    //Main app loop
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.running {
            terminal.draw(ui::render)?;
            self.handle_events()?;
        }

        Ok(())
    }

    //Manage user inputs
    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                _ => {}
            }
        }

        Ok(())
    }

    //Quit the app
    fn quit(&mut self) {
        self.running = false;
    }
}
