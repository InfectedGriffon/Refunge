mod befunge;
mod vector;
mod grid;
mod event;
mod arguments;
mod input;
mod stack;
mod pointer;

use std::io;
use clap::Parser;
use std::io::{stdout, Stdout};
use anyhow::{bail, Result};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen};
use ctrlc_handler::CtrlCHandler;
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::arguments::Arguments;
use crate::befunge::Befunge;

fn main() -> Result<()> {
    let args = Arguments::parse();

    if args.quiet {
        let (max_ticks, log_stack) = (args.max_ticks, args.log_stack);
        let mut befunge = Befunge::new(args);
        let c = CtrlCHandler::new();
        let mut ticks = 0u32;
        while !befunge.ended() && c.should_continue() {
            befunge.tick();
            if let Some(max) = max_ticks {
                if ticks > max {break} else {ticks += 1}
            }
        }
        if log_stack {befunge.log_stacks()}
        if let Some(code) = befunge.exit_code {bail!("process created code {}", code)}
        Ok(())
    } else {
        let mut terminal = create_tui()?;
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
        exit_tui(terminal)?;
        if let Some(code) = befunge.exit_code {bail!("process created code {}", code)}
        Ok(())
    }
}
fn create_tui() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout()))?)
}
fn exit_tui(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(terminal.show_cursor()?)
}
