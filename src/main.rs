mod app;
mod game;
mod network;
mod state;
mod tui;
mod ui;

use app::App;
use std::io;
use tui::Tui;

fn main() -> io::Result<()> {
    let mut tui = Tui::new()?;

    let mut app = App::default();

    let result = app.run(&mut tui.terminal);

    Tui::restore()?;

    result
}
