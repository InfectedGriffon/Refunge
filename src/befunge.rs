use std::collections::VecDeque;
use crate::arguments::Arguments;
use crate::event::{Event, EventHandler, KeyHandler, TickHandler};
use crate::grid::FungeGrid;
use crate::pointer::InstructionPointer;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction::Horizontal, Layout};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use std::fs::read_to_string;
use std::io::Stdout;
use crate::{key, vector};
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers, KeyEventKind};

#[derive(Default)]
pub struct Befunge {
    /// the grid that is being traversed
    grid: FungeGrid,
    /// ip running around executing commands
    ip_list: VecDeque<InstructionPointer>,
    /// output text produced by , and .
    out: String,

    /// toggled by pressing p
    paused: bool,
    /// how far down the grid we've scrolled
    grid_scroll: (u16, u16),
    /// scrolling for output text
    output_scroll: u16,

    /// stored command line arguments
    args: Arguments,
    /// global events
    events: EventHandler,
    /// tickspeed handling
    ticks: TickHandler,
    /// key input
    key_events: KeyHandler,
}
impl Befunge {
    /// create a new befunge simulation
    pub fn new(args: Arguments) -> Befunge {
        let paused = args.paused;
        let grid = FungeGrid::new(read_to_string(&args.file).expect("failed to read file"));
        let ip_list = [InstructionPointer::new(grid.start_pos(args.script), vector::EAST, 0)].into();
        Befunge { grid, ip_list, paused, args, ..Default::default() }
    }
    /// step forward once and run whatever char we're standing on
    pub fn tick(&mut self) {
        for ip in self.ip_list.iter_mut() {
            if ip.dead {continue}
            if !ip.first_tick {ip.walk(&self.grid)}
            let c = self.grid.char_at(ip.pos);
            if ip.string_mode {
                match c {
                    '"' => {ip.string_mode = false}
                    ' ' => {
                        while self.grid.char_at(ip.pos) == ' ' {
                            ip.walk(&self.grid);
                        }
                        ip.walk_reverse(&self.grid);
                        ip.push(32);
                    }
                    _ => ip.push(c as i32),
                }
            } else {
                ip.command(c, &mut self.grid, self.events.sender.clone(), &mut self.out, self.args.quiet);
            }
            if ip.first_tick {ip.first_tick = false}
        }
        match self.events.next() {
            Some(Event::Spawn(ip)) => self.ip_list.push_front(ip),
            Some(Event::Kill) => for ip in self.ip_list.iter_mut() { ip.dead = true },
            None => {}
        }
    }
    /// reset everything
    pub fn restart(&mut self) {
        self.grid.reset();
        self.ip_list = [InstructionPointer::new(self.grid.start_pos(self.args.script), vector::EAST, 0)].into();
        self.out.clear();
        self.paused = self.args.paused;
    }

    /// is there a tick available
    pub fn has_tick(&self) -> bool {
        self.ticks.has_tick()
    }
    /// handle key input for scrolling, pausing, etc
    pub fn handle_key_events(&mut self) -> bool {
        if let Ok(event) = self.key_events.next() {
            match event {
                key!('.') => self.ticks.speed_up(),
                key!(',') => self.ticks.slow_down(),
                key!(Right) if self.paused => self.tick(),
                key!('p') => self.paused = !self.paused,
                key!('h') => self.scroll_grid_left(),
                key!('j') => self.scroll_grid_down(),
                key!('k') => self.scroll_grid_up(),
                key!('l') => self.scroll_grid_right(),
                key!('i') => self.scroll_output_up(),
                key!('o') => self.scroll_output_down(),
                key!('r') => self.restart(),
                key!('q') if self.ended() => return true,
                key!(ctrl;'c') => return true,
                _ => {}
            }
        }
        false
    }
    /// is the sim paused
    pub fn paused(&self) -> bool {
        self.paused
    }
    /// is the sim at the end
    pub fn ended(&self) -> bool {
        self.ip_list.iter().all(|ip|ip.dead)
    }

    /// render the grid, stack, output, and message
    pub fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>) {
        let grid_width = (self.grid.width() as u16+2).clamp(20, 80);
        let grid_height = (self.grid.height() as u16+2).clamp(9, 25);
        let output_height = textwrap::wrap(&self.out, grid_width as usize-2).len() as u16+2;
        // let stack_height = (grid_height+output_height).max(self.stacks[0].len() as u16+2);
        let chunks = Layout::new()
            .constraints(vec![Constraint::Length(grid_width),Constraint::Length(8),Constraint::Length(8),Constraint::Min(0)])
            .direction(Horizontal)
            .split(f.size());
        let column_a = Layout::new()
            .constraints(vec![Constraint::Length(grid_height),Constraint::Length(output_height),Constraint::Min(0)])
            .split(chunks[0]);
        let column_b = Layout::new()
            .constraints(vec![/*Constraint::Length(stack_height),*/Constraint::Min(1)])
            .split(chunks[1]);

        let output = Paragraph::new(self.out.clone())
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Output"))
            .scroll((self.output_scroll, 0));
        // let stack = self.stacks[0].render(if self.stacks.len()>  1 {"TOSS"} else if self.stacks[0].queue_mode {"Queue"} else {"Stack"});

        f.render_widget(self.grid.clone().highlights(self.ip_list.clone()), column_a[0]);
        // f.render_widget(self.grid.render(self.ip.pos).scroll(self.grid_scroll), column_a[0]);
        f.render_widget(output, column_a[1]);
        if self.ended() {f.render_widget(Paragraph::new("sim ended.\npress r to restart,\nor q to exit."), column_a[2])}
        // f.render_widget(stack, column_b[0]);
        // if self.stacks.len() > 1 {
        //     f.render_widget(
        //         self.stacks[1].render("SOSS"),
        //         Layout::new().constraints(vec![Constraint::Length(self.stacks[1].len()as u16+2),Constraint::Min(0)]).split(chunks[2])[0]
        //     )
        // }
        if self.paused {f.render_widget(Paragraph::new("paused"), /*column_b[1]*/column_b[0])}
    }

    pub fn scroll_grid_up(&mut self) { self.grid_scroll.0 = self.grid_scroll.0.saturating_sub(1) }
    pub fn scroll_grid_down(&mut self) { self.grid_scroll.0 += 1 }
    pub fn scroll_grid_left(&mut self) { self.grid_scroll.1 = self.grid_scroll.1.saturating_sub(1) }
    pub fn scroll_grid_right(&mut self) { self.grid_scroll.1 += 1 }
    pub fn scroll_output_up(&mut self) { self.output_scroll = self.output_scroll.saturating_sub(1) }
    pub fn scroll_output_down(&mut self) { self.output_scroll += 1 }
}
