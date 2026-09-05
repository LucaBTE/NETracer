use crossterm::event::Event;
use futures_util::future::LocalBoxFuture;
use ratatui::{Frame, layout::Rect};

///A tab compiled into the application.
pub trait Component {
    fn id(&self) -> &'static str;

    fn title(&self) -> &'static str;

    fn help(&self) -> &'static str;

    ///Return true when the component consumes the input.
    fn handle_event(&mut self, _event: &Event) -> bool {
        false
    }

    ///Update background results, including while the tab is hidden.
    fn update(&mut self) {}

    fn render(&mut self, frame: &mut Frame, area: Rect);

    ///Clear mouse interaction areas when hidden or resized.
    fn reset_layout(&mut self) {}

    ///Finish background cleanup before restoring the terminal.
    fn shutdown(&mut self) -> LocalBoxFuture<'_, ()> {
        Box::pin(async {})
    }
}
