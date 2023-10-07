mod befunge;
mod vector;
mod grid;
mod event;
mod arguments;
mod input;
mod stack;
mod pointer;

use clap::Parser;
use std::io::stdout;
use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen};
use ctrlc_handler::CtrlCHandler;
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::arguments::Arguments;
use crate::befunge::Befunge;

fn main() -> Result<()> {
    let args = Arguments::parse();

    // quiet mode (no display)
    if args.quiet {
        let (max_ticks, print_stack) = (args.max_ticks, args.print_stack);
        let mut befunge = Befunge::new(args);
        let c = CtrlCHandler::new();
        let mut ticks = 0u32;
        while !befunge.ended() && c.should_continue() {
            befunge.tick();
            if let Some(max) = max_ticks {
                if ticks > max {break} else {ticks += 1}
            }
        }
        if print_stack {befunge.print_stacks()}
        Ok(())
    } else {
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
            if befunge.has_tick() && !befunge.paused() {befunge.tick()}
            if befunge.handle_key_events() {break}
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }
}
