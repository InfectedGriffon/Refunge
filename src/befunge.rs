use std::fs::read_to_string;
use std::io::Stdout;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::Frame;
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::widgets::Wrap;
use ratatui::layout::Direction::Horizontal;
use crate::arguments::Arguments;
use crate::grid::FungeGrid;
use crate::direction::Direction::*;
use crate::event::{Event, EventHandler};
use crate::befunge::FungeState::*;
use anyhow::Result;
use crate::input::take_input_parse;

#[derive(PartialEq, Debug, Default)]
enum FungeState {
    #[default]
    Started,
    Running,
    InputtingNum,
    InputtingChar,
    Ended
}

#[derive(Default)]
pub struct Befunge {
    /// the grid that is being traversed
    grid: FungeGrid,
    /// the main data stack used by the sim
    stack: Vec<u32>,
    /// output text produced by , and .
    out: String,

    /// what's going on in there
    state: FungeState,
    /// this is separate from the state for reasons™
    str_mode: bool,
    /// toggled by pressing p
    paused: bool,
    /// skip the next instruction
    skip_next: bool,

    /// stored command line arguments
    args: Arguments,
    /// ticks, key input, and speed control
    events: EventHandler,
    /// store the input before sending
    input: String,
}
impl Befunge {
    /// create a new befunge simulation
    pub fn new(args: Arguments) -> Befunge {
        let paused = args.paused;
        let grid = FungeGrid::new(read_to_string(&args.file).expect("failed to read file"));
        Befunge { grid, paused, args, ..Default::default() }
    }
    /// step forward once and run whatever char we're standing on
    pub fn tick(&mut self) {
        // when at the very beginning, do stuff THEN move
        if !self.ended() && !self.first_tick() {
            self.grid.walk();
        }

        let c = self.grid.current_char();
        if self.str_mode && c != '"' {
            self.push(c as u32);
        } else if !self.skip_next {
            self.command(c);
        } else {
            self.skip_next = false;
        }

        // once ticked a single time, go into normal running
        if self.first_tick() {self.state = Running}
    }
    /// reset everything
    pub fn restart(&mut self) {
        self.grid.reset();
        self.stack.clear();
        self.out.clear();
        self.state = Started;
        self.paused = self.args.paused;
        self.str_mode = false;
        self.skip_next = false;
        self.input.clear();
    }
    /// pause/unpause the sim
    pub fn pause(&mut self) {
        self.paused = !self.paused;
    }
    /// readonly output because exposing fields is gross
    pub fn output(&self) -> String {self.out.clone()}

    /// are we on the first tick of the sim
    pub fn first_tick(&self) -> bool {self.state == Started}
    /// is the sim not paused or at the end
    pub fn running(&self) -> bool {!self.paused && (self.state == Running || self.state == Started)}
    /// is the sim paused
    pub fn paused(&self) -> bool {self.paused && self.state != InputtingNum && self.state != InputtingChar}
    /// is the sim at the end
    pub fn ended(&self) -> bool {self.state == Ended}
    /// whether the sim is taking input from an &
    pub fn inputting_num(&self) -> bool {self.state == InputtingNum}
    /// whether the sim is taking input from a ~
    pub fn inputting_char(&self) -> bool {self.state == InputtingChar}

    /// render the grid, stack, output, and message
    pub fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>) {
        let main_width = (self.grid.width() as u16 + 2).max(32);
        let output_height = self.out.len() as u16 / (main_width-2) + 3;
        let grid_height = self.grid.height() as u16 + 2;
        let stack_height = (output_height + grid_height).max(self.stack.len() as u16 + 2);

        let main_layout = Layout::new()
            // main area / stack
            .constraints(vec![Constraint::Length(main_width), Constraint::Length(8), Constraint::Min(1)])
            .direction(Horizontal)
            .split(f.size());
        let vertical_split = Layout::new()
            // grid / output / message
            .constraints(vec![Constraint::Length(grid_height), Constraint::Length(output_height), Constraint::Min(1)])
            .split(main_layout[0]);
        let stack_layout = Layout::new()
            .constraints(vec![Constraint::Length(stack_height), Constraint::Min(1)])
            .split(main_layout[1]);

        let stack = Paragraph::new(self.stack.iter().rev().map(|n| Line::from(n.to_string())).collect::<Vec<Line>>())
            .block(Block::default().borders(Borders::ALL).title("Stack"));
        let output = Paragraph::new(self.out.clone()).wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Output"));

        f.render_widget(stack, stack_layout[0]);
        f.render_widget(self.grid.render(), vertical_split[0]);
        f.render_widget(output, vertical_split[1]);
        f.render_widget(self.render_message(), vertical_split[2]);
    }
    /// special messages for important events, such as inputting or the sim ending
    pub fn render_message(&self) -> Paragraph {
        let text = match self.state {
            InputtingChar => format!("input char: {}", self.input),
            InputtingNum => format!("input num: {}", self.input),
            Ended => "sim ended.\npress r to restart or q to exit.".to_string(),
            _ if self.str_mode && self.paused => "(string mode, paused)".to_string(),
            _ if self.str_mode => "(string mode)".to_string(),
            _ if self.paused => "(paused)".to_string(),
            _ => String::new(),
        };
        Paragraph::new(text)
    }

    // TODO CLEAN UP INPUT SYSTEM

    /// input a character and go back to normal running state
    pub fn input_char(&mut self, c: char) {
        self.push(c as u32);
        self.state = Running;
        self.input.clear();
    }
    /// input char from quiet mode
    pub fn input_char_quiet(&mut self) {
        self.input_char(take_input_parse::<char>("input char:").unwrap());
    }
    /// add a single digit to number input
    pub fn add_digit(&mut self, c: char) {
        self.input.push(c);
    }
    /// finalize number input
    pub fn input_num(&mut self) {
        self.push(self.input.parse().unwrap_or(0));
        self.state = Running;
        self.input.clear();
    }
    /// input num from quiet mode
    pub fn input_num_quiet(&mut self) {
        self.input = take_input_parse::<u32>("input num:").unwrap().to_string();
        self.input_num();
    }

    /// next key press or tick from event handler
    pub fn next_event(&self) -> Result<Event> {
        self.events.next()
    }
    /// speed up the simulation
    pub fn speed_up(&mut self) {
        self.events.speed_up();
    }
    /// slow down the simulation
    pub fn slow_down(&mut self) {
        self.events.slow_down();
    }

    /// get the top number from the stack or zero
    pub fn pop(&mut self) -> u32 {
        self.stack.pop().unwrap_or(0)
    }
    /// push a number onto the top of the stack
    pub fn push(&mut self, n: u32) {
        self.stack.push(n)
    }
    /// run the instruction from a given character
    pub fn command(&mut self, c: char) {
        match c {
            // integers
            '0'..='9' => self.push(c.to_digit(10).unwrap()),
            // math
            '+' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(x.saturating_add(y))
            }
            '-' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(y.saturating_sub(x));
            }
            '*' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(x.saturating_mul(y))
            }
            '/' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(y / x)
            }
            '%' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(y % x)
            }
            '`' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(if y > x {1} else {0})
            }
            // gambits
            ':' => {
                let n = self.pop();
                self.push(n);
                self.push(n);
            }
            '!' => {
                let n = self.pop();
                self.push(if n == 0 {1} else {0})
            }
            '\\' => {
                let x = self.pop();
                let y = self.pop();
                self.push(x);
                self.push(y);
            }
            '$' => {self.pop();}
            // input/output
            '.' => {
                let n = self.pop();
                self.out.push_str(&n.to_string());
            }
            ',' => {
                let n = char::from_u32(self.pop()).unwrap_or(' ');
                self.out.push(n);
            }
            '&' => self.state = InputtingNum,
            '~' => self.state = InputtingChar,
            // movement
            '^' => self.grid.face(Up),
            'v' => self.grid.face(Down),
            '>' => self.grid.face(Right),
            '<' => self.grid.face(Left),
            '?' => self.grid.face(rand::random()),
            '_' => if self.pop() == 0 {self.grid.face(Right)} else {self.grid.face(Left)},
            '|' => if self.pop() == 0 {self.grid.face(Down)} else {self.grid.face(Up)},
            // misc
            '"' => self.str_mode = !self.str_mode,
            '#' => self.skip_next = true,
            'g' => {
                let (x, y) = (self.pop(), self.pop());
                let c = self.grid.char_at(x as usize, y as usize);
                self.push(c as u32)
            },
            'p' => {
                let (x, y) = (self.pop() as usize, self.pop() as usize);
                let c = char::from_u32(self.pop()).unwrap_or('\x00');
                if self.args.expand {
                    self.grid.set_char_or_expand(x, y, c);
                } else {
                    self.grid.set_char(x, y, c);
                }
            },
            '@' => self.state = Ended,
            ' ' => { /* space = no-op */ }
            c => {
                self.state = Ended;
                panic!("unknown character {} at {:?}", c, self.grid.pos());
            }
        }
    }
}
