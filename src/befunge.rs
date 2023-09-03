use crate::arguments::Arguments;
use crate::data::FungeData;
use crate::direction::Direction::*;
use crate::event::{Event, EventHandler};
use crate::grid::FungeGrid;
use crate::input::take_input_parse;
use crate::state::{self, FungeState, MoveType, OnTick};
use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction::Horizontal, Layout};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use std::fs::read_to_string;
use std::io::Stdout;

#[derive(Default)]
pub struct Befunge {
    /// the grid that is being traversed
    grid: FungeGrid,
    /// simultaneously a stack and a queue
    data: FungeData,
    /// output text produced by , and .
    out: String,

    /// what's going on in there
    state: FungeState,
    /// toggled by pressing p
    paused: bool,

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
        match self.state.movement() {
            MoveType::Halted => { /* do not walk */}
            MoveType::Forward => self.grid.walk(),
            MoveType::Reverse => self.grid.walk_reverse(),
        }

        let c = self.grid.current_char();
        match self.state.action() {
            OnTick::Instruction => self.command(c),
            OnTick::StringPush if c != '"' => self.push(c as i32),
            _ => {}
        }

        if self.state.is_over(c) {
            self.state = state::RUNNING;
        } else {
            self.state.tick();
        }
    }
    /// reset everything
    pub fn restart(&mut self) {
        self.grid.reset();
        self.data.clear();
        self.out.clear();
        self.state = state::STARTED;
        self.paused = self.args.paused;
        self.input.clear();
    }
    /// pause/unpause the sim
    pub fn pause(&mut self) {
        self.paused = !self.paused;
    }
    /// readonly output because exposing fields is gross
    pub fn output(&self) -> String {
        self.out.clone()
    }

    /// is the sim not paused or at the end
    pub fn running(&self) -> bool {
        !self.paused && !self.state.is_end()
    }
    /// is the sim paused
    pub fn paused(&self) -> bool {
        self.paused
    }
    /// is the sim at the end
    pub fn ended(&self) -> bool {
        self.state.is_end()
    }
    /// whether the sim is taking input from an &
    pub fn inputting_num(&self) -> bool {
        self.state.inputting_num()
    }
    /// whether the sim is taking input from a ~
    pub fn inputting_char(&self) -> bool {
        self.state.inputting_char()
    }

    /// render the grid, data, output, and message
    pub fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>) {
        let main_width = (self.grid.width() as u16 + 2).max(32);
        let output_height = self.out.len() as u16 / (main_width - 2) + 3;
        let grid_height = self.grid.height() as u16 + 2;
        let data_height = (output_height + grid_height).max(self.data.len() as u16 + 2);

        let main_layout = Layout::new()
            // main area / data
            .constraints(vec![
                Constraint::Length(main_width), // vertical_split
                Constraint::Length(8),          // data_layout
                Constraint::Min(1),             // padding
            ])
            .direction(Horizontal)
            .split(f.size());
        let vertical_split = Layout::new()
            // grid / output / message
            .constraints(vec![
                Constraint::Length(grid_height),   // grid
                Constraint::Length(output_height), // output
                Constraint::Min(1),                // padding
            ])
            .split(main_layout[0]);
        let data_layout = Layout::new()
            .constraints(vec![Constraint::Length(data_height), Constraint::Min(1)])
            .split(main_layout[1]);

        let output = Paragraph::new(self.out.clone())
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Output"));

        f.render_widget(self.data.render(), data_layout[0]);
        f.render_widget(self.grid.render(), vertical_split[0]);
        f.render_widget(output, vertical_split[1]);
        f.render_widget(self.state.render_message(&self.input), vertical_split[2]);
        if self.paused {
            f.render_widget(Paragraph::new("paused"), data_layout[1]);
        }
    }

    // TODO CLEAN UP INPUT SYSTEM

    /// input a character and go back to normal running state
    pub fn input_char(&mut self, c: char) {
        self.push(c as i32);
        self.state = state::RUNNING;
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
        self.state = state::RUNNING;
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

    /// get the top number from the stack or first number in the queue
    pub fn pop(&mut self) -> i32 {
        self.data.pop()
    }
    /// get the top value from the stack as a character
    pub fn pop_char(&mut self) -> char {
        char::from_u32(self.data.pop() as u32).unwrap_or(' ')
    }
    /// push a number onto the top of the stack or end of the queue
    pub fn push(&mut self, n: i32) {
        self.data.push(n)
    }
    /// run the instruction from a given character
    pub fn command(&mut self, c: char) {
        match c {
            // integers
            '0'..='9'|'a'..='f' => self.push(c.to_digit(16).unwrap() as i32),
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
            'q' => self.data.queue_mode(),
            's' => self.data.stack_mode(),
            'l' => {
                let n = self.pop();
                self.data.permute(n as usize);
            }
            // input/output
            '.' => {
                let n = self.pop();
                self.out.push_str(&n.to_string());
            }
            ',' => {
                let n = self.pop_char();
                self.out.push(n);
            }
            '&' => self.state = state::INPUTTING_NUM,
            '~' => self.state = state::INPUTTING_CHAR,
            // movement
            '^' => self.grid.face(Up),
            'v' => self.grid.face(Down),
            '>' => self.grid.face(Right),
            '<' => self.grid.face(Left),
            '?' => self.grid.face(rand::random()),
            '_' => if self.pop() == 0 {self.grid.face(Right)} else {self.grid.face(Left)},
            '|' => if self.pop() == 0 {self.grid.face(Down)} else {self.grid.face(Up)},
            'r' => self.grid.turn_reverse(),
            '[' => self.grid.turn_left(),
            ']' => self.grid.turn_right(),
            'w' => {
                let (b, a) = (self.pop(), self.pop());
                if a < b {
                    self.grid.turn_right()
                } else if a > b {
                    self.grid.turn_left()
                };
            }
            'j' => {
                let n = self.pop();
                if n < 0 {
                    self.state = state::SKIP_N_REV(n.abs() as u8);
                } else if n > 0 {
                    self.state = state::SKIP_N(n.abs() as u8);
                }
            }
            // misc
            '"' => self.state = state::STRING_MODE,
            '\'' => self.state = state::CHAR_FETCH,
            '#' => self.state = state::SKIP_NEXT,
            ';' => self.state = state::SKIP_UNTIL,
            'g' => {
                let (x, y) = (self.pop(), self.pop());
                let c = self.grid.char_at(x as usize, y as usize);
                self.push(c as i32)
            },
            'p' => {
                let (x, y, c) = (self.pop() as usize, self.pop() as usize, self.pop_char());
                if self.args.expand {
                    self.grid.set_char_or_expand(x, y, c);
                } else {
                    self.grid.set_char(x, y, c);
                }
            },
            '@' => self.state = state::ENDED,
            ' ' => { /* space = no-op */ }
            c => {
                self.state = state::ENDED;
                panic!("unknown character {} at {:?}", c, self.grid.pos());
            }
        }
    }
}
