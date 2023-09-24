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
use std::path::Path;
use std::process::Command;
use chrono::{Datelike, Timelike};
use crate::vector;
use crate::vector::FungeVector;

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
        let ip = InstructionPointer::new(grid.start_pos(args.script));
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

        f.render_widget(self.grid.render(self.ip.pos).scroll(self.grid_scroll), column_a[0]);
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
    /// pop a vector off of the stack in the order y, x
    pub fn pop_vector(&mut self) -> FungeVector {
        let (y, x) = (self.pop(), self.pop());
        FungeVector(x, y)
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
    pub fn push_vector(&mut self, pos: FungeVector) {
        self.push(pos.0);
        self.push(pos.1);
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
        self.grid.char_at(self.ip.pos)
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
                let c = self.grid.char_at(self.grid.cell_ahead_ip(self.ip));
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
            '<' => self.ip.delta = vector::WEST,
            '=' => {
                let cmd = self.pop_0gnirts();
                self.push(Command::new("cmd.exe")
                    .args(vec!["/c", &cmd])
                    .status()
                    .expect("failed to execute")
                    .code()
                    .unwrap_or_default());
            }
            '>' => self.ip.delta = vector::EAST,
            '?' => self.ip.delta = rand::random(),
            '@' => self.state = state::ENDED,
            // A-Z => todo
            '[' => self.ip.delta.turn_left(),
            '\\' => { let pos = FungeVector(self.pop(), self.pop()); self.push_vector(pos) }
            ']' => self.ip.delta.turn_right(),
            '^' => self.ip.delta = vector::NORTH,
            '_' => if self.pop() == 0 {self.ip.delta = vector::EAST } else {self.ip.delta = vector::WEST },
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
                let c = self.grid.char_at(FungeVector(x, y));
                self.push(c as i32)
            },
            // trefunge only: h
            'i' => {
                let filename = self.pop_0gnirts();
                let flags = self.pop();
                let pos = self.pop_vector();
                if !Path::new(&filename).exists() {
                    self.ip.delta.invert()
                } else {
                    let text = read_to_string(filename).unwrap_or_default();
                    self.grid.place(text, pos, flags & 1 != 0);
                }
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

                let c = self.grid.runnable_char_ahead(self.ip.pos, self.ip.delta);
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
                let path = Path::new(&filename);
                let flags = self.pop();
                let v_a = self.pop_vector();
                let v_b = self.pop_vector();
                let mut text = self.grid.read_from(v_a, v_b);
                if flags & 1 != 0 {
                    text = text.lines().map(|l| l.strip_suffix("\r\n").or(l.strip_suffix("\n")).unwrap_or(l)).collect();
                    text = text.trim_end().to_string();
                }
                if path.exists() && !path.metadata().unwrap().permissions().readonly() {
                    File::open(path).unwrap().write_all(text.as_bytes()).unwrap();
                } else if !path.exists() {
                    File::create(path).unwrap().write_all(text.as_bytes()).unwrap();
                } else {
                    self.ip.delta.invert();
                }
            }
            'p' => {
                let pos = self.pop_vector();
                let c = self.pop_char();
                self.grid.set_char(pos + self.ip.offset, c, self.args.expand);
            },
            // todo: q
            'r' => self.ip.delta.invert(),
            's' => {
                let c = self.pop_char();
                let pos = self.grid.cell_ahead_ip(self.ip);
                self.grid.set_char(pos, c, false);
                self.walk();
            },
            // todo: t
            'u' => {
                if self.stacks.len() == 1 { return self.ip.delta.invert() }
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
            'v' => self.ip.delta = vector::SOUTH,
            'w' => {
                let (b, a) = (self.pop(), self.pop());
                if a < b {
                    self.ip.delta.turn_left()
                } else if a > b {
                    self.ip.delta.turn_right()
                };
            }
            'x' => self.ip.delta = self.pop_vector(),
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
                    Box::new(|b| b.push_vector(b.ip.pos)),
                    // 11: delta
                    Box::new(|b| b.push_vector(b.ip.delta)),
                    // 12: storage offset
                    Box::new(|b| b.push_vector(b.ip.offset)),
                    // 13: least point
                    Box::new(|b| b.push_vector(vector::ORIGIN)),
                    // 14: greatest point
                    Box::new(|b| b.push_vector(FungeVector(b.grid.width() as i32, b.grid.height() as i32+1))),
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
                self.ip.offset = self.ip.pos + self.ip.delta;
            }
            '|' => if self.pop() == 0 {self.ip.delta = vector::SOUTH } else {self.ip.delta = vector::NORTH },
            '}' => {
                if self.stacks.len() == 1 { return self.ip.delta.invert() }
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
            _ => if !self.args.ignore { self.ip.delta.invert() },
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