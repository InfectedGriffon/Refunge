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
use std::fs::{File, read_to_string};
use std::io::{Stdout, Write};
use crate::pointer::InstructionPointer;

#[derive(Default)]
pub struct Befunge {
    /// the grid that is being traversed
    grid: FungeGrid,
    /// ip running around executing commands
    ip: InstructionPointer,
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
        let ip = InstructionPointer::new(0, grid.start_pos(args.script).1);
        Befunge { grid, ip, paused, args, ..Default::default() }
    }
    /// step forward once and run whatever char we're standing on
    pub fn tick(&mut self) {
        match self.state.movement() {
            MoveType::Halted => { /* do not walk */}
            MoveType::Forward => self.ip.walk(self.grid.width()-1, self.grid.height()-1),
            MoveType::Reverse => self.ip.walk_reverse(self.grid.width()-1, self.grid.height()-1),
        }

        let c = self.grid.char_at(self.ip.x, self.ip.y);
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
        self.ip.reset();
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
        f.render_widget(self.grid.render(self.ip.x, self.ip.y), vertical_split[0]);
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
    /// get a null-terminated string from the stack
    pub fn pop_0gnirts(&mut self) -> String {
        let mut output = String::new();
        loop {
            let c = self.pop();
            if c == 0 {return output}
            output.push(char::from_u32(c as u32).unwrap_or(' '));
        };
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
            'l' => {
                let n = self.pop();
                self.data.permute(n as usize);
            }
            'k' => {
                let n = self.pop();
                if n == 0 {self.state = state::SKIP_NEXT; return}

                let c = self.grid.char_ahead_ip(self.ip);
                if c != ' ' && c != ';' {
                    for _ in 0..n {
                        self.command(c);
                    }
                }
            }
            'n' => self.data.clear(),
            // input/output
            '.' => {
                let n = self.pop();
                self.out.push_str(&format!("{n} "));
            }
            ',' => {
                let n = self.pop_char();
                self.out.push(n);
            }
            '&' => self.state = state::INPUTTING_NUM,
            '~' => self.state = state::INPUTTING_CHAR,
            'i' => {
                let s = self.pop_0gnirts();
                let flag = self.pop(); // just the one flag
                let (y, x) = (self.pop() as usize, self.pop() as usize);
                let mut text = read_to_string(s).unwrap_or_default();
                if flag & 1 == 1 {text.retain(|c| !['\r','\n'].contains(&c))};
                self.grid.place(text, x, y);
            }
            'o' => {
                let filename = self.pop_0gnirts();
                let _ = self.pop();
                let (y_a, x_a) = (self.pop() as usize, self.pop() as usize);
                let (y_b, x_b) = (self.pop() as usize, self.pop() as usize);
                let content = self.grid.read_from(x_a, y_a, x_b, y_b);
                if let Ok(mut file) = File::open(filename) {
                    write!(file, "{content}").unwrap_or_else(|_| self.ip.turn_reverse());
                }
                // TODO DEAL WITH FLAG
                // "if the least significant bit of the flags cell is high,
                // `o` treats the file as a linear text file;
                // that is, any spaces before each EOL, and any EOLs before the EOF, are not written out.
                // The resulting text file is identical in appearance and takes up less storage space."
            }
            // movement
            '^' => self.ip.face(Up),
            'v' => self.ip.face(Down),
            '>' => self.ip.face(Right),
            '<' => self.ip.face(Left),
            '?' => self.ip.face(rand::random()),
            '_' => if self.pop() == 0 {self.ip.face(Right)} else {self.ip.face(Left)},
            '|' => if self.pop() == 0 {self.ip.face(Down)} else {self.ip.face(Up)},
            'r' => self.ip.turn_reverse(),
            '[' => self.ip.turn_left(),
            ']' => self.ip.turn_right(),
            'w' => {
                let (b, a) = (self.pop(), self.pop());
                if a < b {
                    self.ip.turn_right()
                } else if a > b {
                    self.ip.turn_left()
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
            's' => {
                let c = self.pop_char();
                let (x, y) = self.grid.cell_ahead_ip(self.ip);
                self.grid.set_char(x, y, c);
            },
            '#' => self.state = state::SKIP_NEXT,
            ';' => self.state = state::SKIP_UNTIL,
            'g' => {
                let (y, x) = (self.pop(), self.pop());
                let c = self.grid.char_at(x as usize, y as usize);
                self.push(c as i32)
            },
            'p' => {
                let (y, x, c) = (self.pop() as usize, self.pop() as usize, self.pop_char());
                if self.args.expand {
                    self.grid.set_char_or_expand(x, y, c);
                } else {
                    self.grid.set_char(x, y, c);
                }
            },
            '@' => self.state = state::ENDED,
            ' ' => { /* space = no-op */ }
            _ => if !self.args.ignore { self.ip.turn_reverse() },
        }
    }
}
