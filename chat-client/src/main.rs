use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, thread, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    widgets::{Block, Borders},
    Frame, Terminal,
};

fn draw<B: Backend>(f: &mut Frame<B>) {
    let size = f.size();
    let block = Block::default().title("Block").borders(Borders::ALL);
    f.render_widget(block, size);
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(draw)?;

    thread::sleep(Duration::from_secs(5));

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    //println!("Hello, world!");

    Ok(())
}
