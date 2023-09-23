mod befunge;
mod vector;
mod grid;
mod event;
mod arguments;
mod input;
mod stack;
mod state;
mod pointer;

use clap::Parser;
use std::io::stdout;
use anyhow::Result;
use crossterm::execute;
use crossterm::event::{KeyEvent, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen};
use ctrlc_handler::CtrlCHandler;
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::arguments::Arguments;
use crate::befunge::Befunge;
use crate::event::Event;

fn main() -> Result<()> {
    let args = Arguments::parse();

    // quiet mode (no display)
    if args.quiet {
        let max_ticks = args.max_ticks;
        let mut befunge = Befunge::new(args);
        let c = CtrlCHandler::new();
        let mut ticks = 0u32;
        'main: while !befunge.ended() && c.should_continue() {
            if befunge.inputting_char() {
                befunge.input_char_quiet();
            } else if befunge.inputting_num() {
                befunge.input_num_quiet();
            } else {
                befunge.tick();
            }
            if let Some(max) = max_ticks {
                if ticks > max {
                    break 'main
                } else {
                    ticks += 1;
                }
            }
        }
        return Ok(());
    }

    // normal tui display
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let jump_ticks = args.jump;
    let mut befunge = Befunge::new(args);

    if let Some(n) = jump_ticks {
        for _ in 0..n { befunge.tick() }
    }

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

            // scrolling
            Event::Key(key!('h')) => befunge.scroll_grid_left(),
            Event::Key(key!('j')) => befunge.scroll_grid_down(),
            Event::Key(key!('k')) => befunge.scroll_grid_up(),
            Event::Key(key!('l')) => befunge.scroll_grid_right(),
            Event::Key(key!('i')) => befunge.scroll_output_up(),
            Event::Key(key!('o')) => befunge.scroll_output_down(),

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
