use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        execute,
        style::Print,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
        cursor::{Hide, Show},
    },
};
use std::io::{self, Stdout};

/// Manages the terminal user interface lifecycle.
pub struct Tui {
    /// Interface for drawing frames to the terminal.
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// Initializes the terminal by entering raw mode and the alternate screen.
    pub fn new() -> io::Result<Self> {
        // enable raw mode to capture keyboard input directly.
        enable_raw_mode()?;

        // enter the alternate screen and enable mouse support protocols.
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            Print("\x1b[?1000h\x1b[?1006h"),
            Hide
        )?;

        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    /// Restores the terminal to its original state before exiting.
    pub fn restore() -> io::Result<()> {
        // disable raw mode to return control to the shell.
        disable_raw_mode()?;

        // exit the alternate screen and disable mouse support.
        execute!(
            io::stdout(),
            LeaveAlternateScreen,
            Print("\x1b[?1000l\x1b[?1006l"),
            Show
        )?;
        Ok(())
    }
}