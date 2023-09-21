use std::env::{args, vars};
use crate::arguments::Arguments;
use crate::stack::FungeStack;
use crate::event::{Event, EventHandler};
use crate::grid::FungeGrid;
use crate::input::take_input_parse;
use crate::state::{self, FungeState, OnTick};
use crate::pointer::InstructionPointer;
use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction::Horizontal, Layout};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use std::fs::{File, read_to_string};
use std::io::{Stdout, Write};
use std::process::Command;
use chrono::{Datelike, Timelike};
use crate::delta;

#[derive(Default)]
pub struct Befunge {
    /// the grid that is being traversed
    grid: FungeGrid,
    /// ip running around executing commands
    ip: InstructionPointer,
    /// list of stacks (0th is SOSS, 1st is TOSS)
    stacks: FungeStack<FungeStack<i32>>,
    /// output text produced by , and .
    out: String,

    /// what's going on in there
    state: FungeState,
    /// toggled by pressing p
    paused: bool,
    /// how far down the grid we've scrolled
    grid_scroll: (u16, u16),
    /// scrolling for output text
    output_scroll: u16,

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
        let stacks = vec![FungeStack::default()].into();
        Befunge { grid, ip, stacks, paused, args, ..Default::default() }
    }
    /// step forward once and run whatever char we're standing on
    pub fn tick(&mut self) {
        if self.state.moving() { self.walk() }

        let c = self.current_char();
        match self.state.action() {
            OnTick::Instruction => self.command(c),
            OnTick::StringPush => match c {
                '"' => {}
                ' ' => {
                    while self.current_char() == ' ' {self.walk()}
                    self.walk_reverse();
                    self.push(32);
                }
                _ => self.push(c as i32),
            },
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
        self.stacks = vec![FungeStack::default()].into();
        self.out.clear();
        self.state = state::STARTED;
        self.paused = self.args.paused;
        self.input.clear();
    }
    /// pause/unpause the sim
    pub fn pause(&mut self) {
        self.paused = !self.paused;
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

    /// render the grid, stack, output, and message
    pub fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>) {
        let grid_width = (self.grid.width() as u16+2).clamp(20, 80);
        let grid_height = (self.grid.height() as u16+2).clamp(9, 25);
        let output_height = textwrap::wrap(&self.out, grid_width as usize-2).len() as u16+2;
        let stack_height = (grid_height+output_height).max(self.stacks[0].len() as u16+2);
        let chunks = Layout::new()
            .constraints(vec![Constraint::Length(grid_width),Constraint::Length(8),Constraint::Length(8),Constraint::Min(0)])
            .direction(Horizontal)
            .split(f.size());
        let column_a = Layout::new()
            .constraints(vec![Constraint::Length(grid_height),Constraint::Length(output_height),Constraint::Min(0)])
            .split(chunks[0]);
        let column_b = Layout::new()
            .constraints(vec![Constraint::Length(stack_height),Constraint::Min(1)])
            .split(chunks[1]);

        let output = Paragraph::new(self.out.clone())
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Output"))
            .scroll((self.output_scroll, 0));
        let stack = self.stacks[0].render(if self.stacks.len()>  1 {"TOSS"} else if self.stacks[0].queue_mode {"Queue"} else {"Stack"});

        f.render_widget(self.grid.render(self.ip.x, self.ip.y).scroll(self.grid_scroll), column_a[0]);
        f.render_widget(output, column_a[1]);
        f.render_widget(self.state.render_message(&self.input).wrap(Wrap{trim:false}), column_a[2]);
        f.render_widget(stack, column_b[0]);
        if self.stacks.len() > 1 {
            f.render_widget(
                self.stacks[1].render("SOSS"),
                Layout::new().constraints(vec![Constraint::Length(self.stacks[1].len()as u16+2),Constraint::Min(0)]).split(chunks[2])[0]
            )
        }
        if self.paused {f.render_widget(Paragraph::new("paused"), column_b[1])}
    }

    pub fn scroll_grid_up(&mut self) { self.grid_scroll.0 = self.grid_scroll.0.saturating_sub(1) }
    pub fn scroll_grid_down(&mut self) { self.grid_scroll.0 += 1 }
    pub fn scroll_grid_left(&mut self) { self.grid_scroll.1 = self.grid_scroll.1.saturating_sub(1) }
    pub fn scroll_grid_right(&mut self) { self.grid_scroll.1 += 1 }
    pub fn scroll_output_up(&mut self) { self.output_scroll = self.output_scroll.saturating_sub(1) }
    pub fn scroll_output_down(&mut self) { self.output_scroll += 1 }


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

    /// get the top number from the stack
    pub fn pop(&mut self) -> i32 {
        self.stacks[0].pop()
    }
    /// get the top value from the stack as a character
    pub fn pop_char(&mut self) -> char {
        char::from_u32(self.pop() as u32).unwrap_or(' ')
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
    /// push a number onto the top of the stack
    pub fn push(&mut self, n: i32) {
        self.stacks[0].push(n)
    }
    /// push a null-terminated string onto the stack
    pub fn push_0gnirts(&mut self, s: String) {
        self.push(0);
        s.chars().rev().for_each(|c| self.push(c as i32));
    }
    // push a 2d coordinate onto the stack
    pub fn push_vector(&mut self, x: i32, y: i32) {
        self.push(x);
        self.push(y);
    }

    /// walk the current ip forward by a space
    pub fn walk(&mut self) {
        self.ip.walk(self.grid.width(), self.grid.height());
    }
    /// walk the current ip backward by a space
    pub fn walk_reverse(&mut self) {
        self.ip.walk_reverse(self.grid.width(), self.grid.height());
    }
    /// character under the current ip
    pub fn current_char(&self) -> char {
        self.grid.char_at(self.ip.x, self.ip.y)
    }

    /// run the instruction from a given character
    pub fn command(&mut self, c: char) {
        match c {
            ' ' => {
                while self.current_char() == ' ' { self.walk() }
                self.walk_reverse();
            }
            '!' => {
                let n = self.pop();
                self.push(if n == 0 {1} else {0})
            }
            '"' => self.state = state::STRING_MODE,
            '#' => self.walk(),
            '$' => {self.pop();}
            '%' => {
                let (x, y) = (self.pop(), self.pop());
                if x == 0 {self.push(0)} else {self.push(y % x)}
            }
            '&' => self.state = state::INPUTTING_NUM,
            '\'' => {
                let (x, y) = self.grid.cell_ahead_ip(self.ip);
                let c = self.grid.char_at(x, y);
                self.push(c as i32);
                self.walk();
            }
            // todo: (
            // todo: )
            '*' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(x.saturating_mul(y))
            }
            '+' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(x.saturating_add(y))
            }
            ',' => {
                let c = self.pop_char();
                if self.args.quiet {
                    print!("{c}")
                } else {
                    self.out.push(c);
                }
            }
            '-' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(y.saturating_sub(x));
            }
            '.' => {
                let n = self.pop();
                if self.args.quiet {
                    print!("{n} ");
                } else {
                    self.out.push_str(&format!("{n} "));
                }
            }
            '/' => {
                let (x, y) = (self.pop(), self.pop());
                if x == 0 {self.push(0)} else {self.push(y / x)}
            }
            '0' => self.push(0),
            '1' => self.push(1),
            '2' => self.push(2),
            '3' => self.push(3),
            '4' => self.push(4),
            '5' => self.push(5),
            '6' => self.push(6),
            '7' => self.push(7),
            '8' => self.push(8),
            '9' => self.push(9),
            ':' => {
                let n = self.pop();
                self.push(n);
                self.push(n);
            }
            ';' => {
                self.walk(); // move off of current semicolon
                while self.current_char() != ';' {self.walk()}
            },
            '<' => self.ip.d = delta::EAST,
            '=' => {
                let cmd = self.pop_0gnirts();
                self.push(Command::new("cmd.exe")
                    .args(vec!["/c", &cmd])
                    .status()
                    .expect("failed to execute")
                    .code()
                    .unwrap_or_default());
            }
            '>' => self.ip.d = delta::WEST,
            '?' => self.ip.d = rand::random(),
            '@' => self.state = state::ENDED,
            // A-Z => todo
            '[' => self.ip.turn_left(),
            '\\' => { let (x, y) = (self.pop(), self.pop()); self.push_vector(x, y) }
            ']' => self.ip.turn_right(),
            '^' => self.ip.d = delta::NORTH,
            '_' => if self.pop() == 0 {self.ip.d = delta::WEST } else {self.ip.d = delta::EAST },
            '`' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(if y > x {1} else {0})
            }
            'a' => self.push(10),
            'b' => self.push(11),
            'c' => self.push(12),
            'd' => self.push(13),
            'e' => self.push(14),
            'f' => self.push(15),
            'g' => {
                let (y, x) = (self.pop(), self.pop());
                if x < 0 || y < 0 { return }
                let c = self.grid.char_at(x as usize, y as usize);
                self.push(c as i32)
            },
            // trefunge only: h
            'i' => {
                let s = self.pop_0gnirts();
                let flag = self.pop(); // just the one flag
                let (y, x) = (self.pop() as usize, self.pop() as usize);
                let mut text = read_to_string(s).unwrap_or_default();
                if flag & 1 == 1 {text.retain(|c| !['\r','\n'].contains(&c))};
                self.grid.place(text, x, y);
            }
            'j' => {
                let n = self.pop();
                if n < 0 {
                    for _ in 0..n.abs() {self.walk_reverse()}
                } else if n > 0 {
                    for _ in 0..n {self.walk()}
                }
            }
            'k' => {
                let n = self.pop();
                if n == 0 {return self.walk()}

                let c = self.grid.runnable_char_ahead(self.ip.x, self.ip.y, self.ip.d);
                for _ in 0..n { self.command(c) }
            }
            'l' => {
                let n = self.pop();
                self.stacks[0].permute(n as usize);
            }
            // trefunge only: m
            'n' => self.stacks[0].clear(),
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
            'p' => {
                let (y, x, c) = (self.pop() + self.ip.offset.1, self.pop() + self.ip.offset.0, self.pop_char());
                if x < 0 || y < 0 { return }
                self.grid.set_char(x as usize, y as usize, c, self.args.expand);
            },
            // todo: q
            'r' => self.ip.turn_reverse(),
            's' => {
                let c = self.pop_char();
                let (x, y) = self.grid.cell_ahead_ip(self.ip);
                self.grid.set_char(x, y, c, false);
                self.walk();
            },
            // todo: t
            'u' => {
                if self.stacks.len() == 1 { return self.ip.turn_reverse() }
                let count = self.pop();
                match count.signum() {
                    1 => {
                        for _ in 0..count {
                            let elem = self.stacks[1].pop();
                            self.push(elem);
                        }
                    }
                    -1 => {
                        for _ in 0..count.abs() {
                            let elem = self.pop();
                            self.stacks[1].push(elem);
                        }
                    }
                    _ => {}
                }
            }
            'v' => self.ip.d = delta::SOUTH,
            'w' => {
                let (b, a) = (self.pop(), self.pop());
                if a < b {
                    self.ip.turn_left()
                } else if a > b {
                    self.ip.turn_right()
                };
            }
            'x' => {
                let (y, x) = (self.pop(), self.pop());
                self.ip.d.x = x;
                self.ip.d.y = y;
            }
            'y' => {
                let n = self.pop();

                let info: Vec<Box<fn(&mut Befunge)>> = vec![
                    // 1: flags getch, =, o, i, no t
                    Box::new(|b| b.push(0b11110)),
                    // 2: bytes per cell
                    Box::new(|b| b.push(std::mem::size_of::<i32>() as i32)),
                    // 3: handprint
                    Box::new(|b| b.push(hexify("RFNG"))),
                    // 4: version number
                    Box::new(|b| b.push(env!("CARGO_PKG_VERSION").replace(".","").parse().unwrap())),
                    // 5: how does "=" work
                    Box::new(|b| b.push(1)),
                    // 6: path separator
                    Box::new(|b| b.push(std::path::MAIN_SEPARATOR as i32)),
                    // 7: dimension
                    Box::new(|b| b.push(2)),
                    // 8: pointer id
                    Box::new(|b| b.push(0)),
                    // 9: team number
                    Box::new(|b| b.push(0)),
                    // 10: pos
                    Box::new(|b| b.push_vector(b.ip.x as i32, b.ip.y as i32)),
                    // 11: delta
                    Box::new(|b| b.push_vector(b.ip.d.x, b.ip.d.y)),
                    // 12: storage offset
                    Box::new(|b| b.push_vector(b.ip.offset.0, b.ip.offset.1)),
                    // 13: least point
                    Box::new(|b| b.push_vector(0, 0)),
                    // 14: greatest point
                    Box::new(|b| b.push_vector(b.grid.width() as i32, b.grid.height() as i32)),
                    // 15: ((year - 1900) * 256 * 256) + (month * 256) + (day of month)
                    Box::new(|b| { let now = chrono::Utc::now(); b.push(((now.year()-1900)*256*256) + (now.month() as i32*256) + now.day() as i32) }),
                    // 16: (hour * 256 * 256) + (minute * 256) + (second)
                    Box::new(|b| { let now = chrono::Utc::now(); b.push(now.hour() as i32*256*256 + now.minute() as i32*256 + now.second() as i32) }),
                    // 17: size of stack-stack
                    Box::new(|b| b.push(b.stacks.len() as i32)),
                    // 18: size of stack
                    Box::new(|b| b.push(b.stacks[0].len() as i32)),
                    // 19: program arguments as 0gnirts, with another nul at end
                    Box::new(|b| b.push_0gnirts(args().collect::<Vec<String>>().join("\x00") + "\x00\x00")),
                    // 20: env vars as key=val 0nigrts, with another null at end
                    Box::new(|b| b.push_0gnirts(vars().map(|(k,v)|format!("{k}={v}")).collect::<Vec<String>>().join("\x00") + "\x00\x00")),
                ];

                match n {
                    ..=0 => info.iter().rev().for_each(|i| i(self)),
                    1..=20 => info[n as usize-1](self),
                    21.. => (0..n-20).for_each(|_|{self.pop();})
                }
            }
            'z' => { /* nop */ }
            '{' => {
                let n = self.pop();
                self.stacks.push_front(FungeStack::default());
                match n.signum() {
                    1 => {
                        let elems: Vec<i32> = (0..n).map(|_|self.stacks[1].pop()).collect();
                        for val in elems.iter().rev() { self.push(*val) }
                    }
                    -1 => for _ in 0..n.abs() { self.stacks[1].push(0) },
                    _ => {}
                }
                self.stacks[1].push(self.ip.offset.0);
                self.stacks[1].push(self.ip.offset.1);
                self.ip.offset = (self.ip.x as i32 + self.ip.d.x, self.ip.y as i32 + self.ip.d.y);
            }
            '|' => if self.pop() == 0 {self.ip.d = delta::SOUTH } else {self.ip.d = delta::NORTH },
            '}' => {
                if self.stacks.len() == 1 { return self.ip.turn_reverse(); }
                let n = self.pop();
                (self.ip.offset.1, self.ip.offset.0) = (self.stacks[1].pop(), self.stacks[1].pop());
                match n.signum() {
                    1 => {
                        let elems: Vec<i32> = (0..n).map(|_|self.pop()).collect();
                        for val in elems.iter().rev() { self.stacks[1].push(*val) }
                    }
                    -1 => for _ in 0..n.abs() { self.stacks[1].pop(); },
                    _ => {}
                }
                self.stacks.pop_front();
            }
            '~' => self.state = state::INPUTTING_CHAR,
            _ => if !self.args.ignore { self.ip.turn_reverse() },
        }
    }
}

/// convert a string into hexadecimal format (for hand/fingerprints)
fn hexify(s: &str) -> i32 {
    let mut hex = 0;
    let mut shift_counter = s.len() * 8;
    for c in s.chars() {
        shift_counter -= 8;
        hex |= (c as i32) << shift_counter;
    }
    hex
}