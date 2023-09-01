mod befunge;
mod direction;
mod grid;
mod event;
mod arguments;
mod input;
mod data;

use clap::Parser;
use std::io::stdout;
use anyhow::Result;
use crossterm::execute;
use crossterm::event::{KeyEvent, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen};
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::arguments::Arguments;
use crate::befunge::Befunge;
use crate::event::Event;

fn main() -> Result<()> {
    let args = Arguments::parse();

    // quiet mode (no display)
    if args.quiet {
        let mut befunge = Befunge::new(args);
        while !befunge.ended() {
            if befunge.inputting_char() {
                befunge.input_char_quiet();
            } else if befunge.inputting_num() {
                befunge.input_num_quiet();
            } else {
                befunge.tick();
            }
        }
        println!("{}", befunge.output());
        return Ok(());
    }

    // normal tui display
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut befunge = Befunge::new(args);

    loop {
        terminal.draw(|f| befunge.render(f))?;

        match befunge.next_event()? {
            // tick befunge
            Event::Tick if befunge.running() => befunge.tick(),

            // input characters
            Event::Key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Release, .. })
                if befunge.inputting_char() => befunge.input_char(c),
            Event::Key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::SHIFT, kind: KeyEventKind::Release, .. })
                if befunge.inputting_char() => befunge.input_char(c.to_ascii_uppercase()),

            // input numbers
            Event::Key(KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::NONE, kind: KeyEventKind::Release, .. })
                if c.is_numeric() && befunge.inputting_num() => befunge.add_digit(c),
            Event::Key(key!(Enter)) if befunge.inputting_num() => befunge.input_num(),

            // speed
            Event::Key(key!('.')) => befunge.speed_up(),
            Event::Key(key!(',')) => befunge.slow_down(),
            Event::Key(key!(Right)) if befunge.paused() => befunge.tick(),
            Event::Key(key!('p')) => befunge.pause(),

            // exit/restart
            Event::Key(key!('r')) => befunge.restart(),
            Event::Key(key!('q')) if befunge.ended() => break,
            Event::Key(key!(ctrl;'c')) => break,

            _ => {}
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
