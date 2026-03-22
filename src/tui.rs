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

/// TUI manager.
pub struct Tui {
    // Terminal interface.
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// Initialize TUI.
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
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

    /// Restore terminal.
    pub fn restore() -> io::Result<()> {
        disable_raw_mode()?;
        execute!(
            io::stdout(),
            LeaveAlternateScreen,
            Print("\x1b[?1000l\x1b[?1006l"),
            Show
        )?;
        Ok(())
    }
}
