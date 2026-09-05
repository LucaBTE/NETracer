use std::io;

use crossterm::{
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
};
use ratatui::DefaultTerminal;

///Owns terminal modes and restores them when dropped.
pub struct TerminalSession {
    terminal: DefaultTerminal,
}

impl TerminalSession {
    pub fn new() -> io::Result<Self> {
        let terminal = ratatui::try_init()?;

        //Extend Ratatui's panic hook to restore input modes too.
        let previous = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |info| {
            let _ = execute!(io::stdout(), DisableMouseCapture, DisableBracketedPaste);

            previous(info);
        }));

        let session = Self { terminal };

        execute!(io::stdout(), EnableMouseCapture, EnableBracketedPaste)?;

        Ok(session)
    }

    pub fn terminal(&mut self) -> &mut DefaultTerminal {
        &mut self.terminal
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), DisableMouseCapture, DisableBracketedPaste);

        ratatui::restore();
    }
}
