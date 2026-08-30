mod app;
mod network;
mod ui;

use std::io;

use app::App;

fn main() -> io::Result<()> {
    //app state
    let mut app = App::new();

    //Ratatui prepares terminal and pass app control
    ratatui::run(|terminal| app.run(terminal))
}
