use std::io;

use netracer::{app::App, tabs, terminal::TerminalSession};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut app = App::new(tabs::builtins())?;
    let mut terminal = TerminalSession::new()?;
    app.run(terminal.terminal()).await
}
