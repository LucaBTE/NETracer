//! Run with: cargo run --example custom_tab
//! A compiled extension: the core event loop and existing tabs do not change.
use std::io;
use netracer::{app::App, component::Component, tabs, terminal::TerminalSession};
use ratatui::{Frame, layout::Rect, widgets::{Block, Borders, Paragraph}};

struct NotesTab;

impl Component for NotesTab {
    fn id(&self) -> &'static str { "notes-example" }
    fn title(&self) -> &'static str { "Notes" }
    fn help(&self) -> &'static str { "Q/Esc: Quit" }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new("A third tab registered without changing App or Ping.")
                .block(Block::default().title(" Extension example ").borders(Borders::ALL)),
            area,
        );
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut components = tabs::builtins();
    components.push(Box::new(NotesTab));
    let mut app = App::new(components)?;
    let mut terminal = TerminalSession::new()?;
    app.run(terminal.terminal()).await
}
