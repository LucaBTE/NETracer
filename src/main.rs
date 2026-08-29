
use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use ratatui::{
    layout::{Alignment, Constraint, Layout},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal,
};

fn main() -> io::Result<()> {
    //runs Ratatui and auto manage setupd and restore terminal view
    ratatui::run(run)
}


fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    //Main TUI event loop

    loop{
        terminal.draw(|frame| {
            //divides view in header, body and footer

            let[header_area, body_area, footer_area] = Layout::vertical([
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(3),
            ])
            .areas(frame.area());
            

            //initialize widgets
            let header = Paragraph::new("NETracer // NETWORK DIAGNOSIS")
                .alignment(Alignment::Center)
                .block(Block::new().borders(Borders::ALL));

            let body = Paragraph::new("\nSYSTEM READ\n\nNetwork diagnostics module awaiting initialization.",
            )
            .alignment(Alignment::Center)
            .block(
                Block::new().title(" NODE STATUS ").borders(Borders::ALL),
            );

            let footer = Paragraph::new("[Q] Quit    [ESC] Exit")
                .alignment(Alignment::Center).block(Block::new().borders(Borders::ALL));

            

            //draws widgets
            frame.render_widget(header, header_area);
            frame.render_widget(body, body_area);
            frame.render_widget(footer, footer_area);

        })?;

        //waits keyboard event

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                break;
            }
        }
    }

    Ok(())
}