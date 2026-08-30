use std::{
    io,
    time::{Duration, Instant},
};

use crate::network::NetworkInfo;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::ui;

pub struct App {
    running: bool,
    started_at: Instant,
    pub network: NetworkInfo,
}

//App functions
impl App {
    //Initialize app state
    pub fn new() -> Self {
        Self {
            running: true,
            started_at: Instant::now(),
            network: NetworkInfo::discover(),
        }
    }

    //Time from NETracer boot
    //&self and not &mut self because uptime doesnt modify App
    pub fn uptime(&self) -> Duration {
        self.started_at.elapsed()
    }

    //Main app loop
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.running {
            terminal.draw(|frame| ui::render(frame, self))?;

            self.handle_events()?;
        }

        Ok(())
    }

    //Manage user inputs
    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(250))?
            && let Event::Key(key) = event::read()?
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
