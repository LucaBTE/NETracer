use crossterm::event::Event;
use futures_util::future::LocalBoxFuture;
use ratatui::{Frame, layout::Rect};

/// A tab compiled into the application. No dependency on App or other tabs.
pub trait Component {
    fn id(&self) -> &'static str;
    fn title(&self) -> &'static str;
    fn help(&self) -> &'static str;

    /// Return true when the component consumes the input.
    fn handle_event(&mut self, _event: &Event) -> bool {
        false
    }

    /// Called for every component, including hidden tabs.
    fn update(&mut self) {}

    fn render(&mut self, frame: &mut Frame, area: Rect);

    /// Clear mouse hit areas when hidden or resized.
    fn reset_layout(&mut self) {}

    /// Await background cleanup before restoring the terminal.
    fn shutdown(&mut self) -> LocalBoxFuture<'_, ()> {
        Box::pin(async {})
    }
}
