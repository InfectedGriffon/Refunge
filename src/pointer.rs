use crate::befunge::InputType;
use crate::event::Event;
use crate::grid::FungeGrid;
use crate::stack::FungeStack;
use crate::stackable::Stackable;
use crate::vector::{directions, FungeVector};
use chrono::{Datelike, Timelike};
use std::env::{args, vars};
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;

macro_rules! stack_op {
    ($ip:expr; $($name:ident),*; $($value:expr),*) => {{
        $( let $name: i32 = $ip.pop(); )*
        $( $ip.push($value); )*
    }};
}

/// an IP that reads from funge-space and performs instructions to its stack
#[derive(Debug, Default, Clone)]
pub struct InstructionPointer {
    pub pos: FungeVector,
    pub delta: FungeVector,
    pub offset: FungeVector,
    pub string_mode: bool,
    pub stacks: FungeStack<FungeStack<i32>>,
    pub id: usize,
    pub dead: bool,
    pub first_tick: bool,
}
impl InstructionPointer {
    /// create a new instruction pointer with specified pos, direction, and id
    pub fn new(pos: FungeVector, delta: FungeVector, id: usize) -> InstructionPointer {
        InstructionPointer {
            pos,
            delta,
            stacks: vec![FungeStack::default()].into(),
            id,
            first_tick: true,
            ..Default::default()
        }
    }

    /// move one space forwards, wrapping around if needed
    pub fn walk(&mut self, grid: &FungeGrid) {
        self.pos.0 = (self.pos.0 + self.delta.0).rem_euclid(grid.width() as i32);
        self.pos.1 = (self.pos.1 + self.delta.1).rem_euclid(grid.height() as i32);
    }
    /// move one space backwards, wrapping around if needed
    pub fn walk_reverse(&mut self, grid: &FungeGrid) {
        self.pos.0 = (self.pos.0 - self.delta.0).rem_euclid(grid.width() as i32);
        self.pos.1 = (self.pos.1 - self.delta.1).rem_euclid(grid.height() as i32);
    }

    /// get the top value from the stack
    pub fn pop(&mut self) -> i32 {
        self.stacks[0].pop()
    }
    /// get the top value from the stack as another type
    pub fn pop_t<T: Stackable>(&mut self) -> T {
        T::pop(&mut self.stacks[0])
    }
    /// push a value onto the top of the stack
    pub fn push<T: Stackable>(&mut self, val: T) {
        T::push(&mut self.stacks[0], val);
    }

    /// execute a Funge-98 instruction based on a given character,
    /// requires access to a grid and external outputs
    pub fn command(
        &mut self,
        c: char,
        grid: &mut FungeGrid,
        sender: mpsc::Sender<Event>,
        out: &mut String,
        quiet: bool,
    ) {
        match c {
            // Space
            ' ' => {
                while grid.char_at(self.pos) == ' ' {
                    self.walk(grid)
                }
                self.command(grid.char_at(self.pos), grid, sender.clone(), out, quiet);
            }
            // Logical Not
            '!' => stack_op!(self; n; if n == 0 {1} else {0}),
            '"' => self.string_mode = true,
            // Trampoline
            '#' => self.walk(grid),
            // Pop
            '$' => stack_op!(self; _del; ),
            // Remainder
            '%' => stack_op!(self; x, y; y.checked_rem(x).unwrap_or_default()),
            // Input Integer
            '&' => sender
                .send(Event::Input(InputType::Number, self.id))
                .unwrap(),
            // Fetch Character
            '\'' => {
                self.walk(grid);
                self.push(grid.char_at(self.pos));
            }
            // '(' { Fingerprints: Load Semantics }
            // ')' { Fingerprints: Unload Semantics }
            // Multiply
            '*' => stack_op!(self; x, y; x.saturating_mul(y)),
            // Add
            '+' => stack_op!(self; x, y; x.saturating_add(y)),
            // Output Character
            ',' => {
                let c: char = self.pop_t();
                if quiet {
                    print!("{c}");
                } else {
                    out.push(c);
                }
            }
            // Subtract
            '-' => stack_op!(self; x, y; y.saturating_sub(x)),
            // Output Integer
            '.' => {
                let n = self.pop();
                if quiet {
                    print!("{n} ");
                } else {
                    out.push_str(&format!("{n} "));
                }
            }
            // Divide
            '/' => stack_op!(self; x, y; y.checked_div(x).unwrap_or_default()),
            // Decimal Literals
            '0'..='9' => stack_op!(self; ; c.to_digit(10).unwrap() as i32),
            // Duplicate
            ':' => stack_op!(self; n; n, n),
            // Jump Over
            ';' => {
                self.walk(grid); // move off of current semicolon
                while grid.char_at(self.pos) != ';' {
                    self.walk(grid);
                }
                self.walk(grid);
                self.command(grid.char_at(self.pos), grid, sender.clone(), out, quiet);
            }
            // Go West
            '<' => self.delta = directions::WEST,
            // Execute
            '=' => {
                let cmd: String = self.pop_t();
                self.push(
                    Command::new("cmd.exe")
                        .args(vec!["/c", &cmd])
                        .status()
                        .expect("failed to execute")
                        .code()
                        .unwrap_or_default(),
                );
            }
            // Go East
            '>' => self.delta = directions::EAST,
            // Go Away
            '?' => self.delta = rand::random(),
            // Stop
            '@' => self.dead = true,
            // 'A'...'Z' { Fingerprints }
            // Turn Left
            '[' => self.delta.turn_left(),
            // Swap
            '\\' => stack_op!(self; x, y; x, y),
            // Turn Right
            ']' => self.delta.turn_right(),
            // Go North
            '^' => self.delta = directions::NORTH,
            // East-West If
            '_' => {
                if self.pop() == 0 {
                    self.delta = directions::EAST
                } else {
                    self.delta = directions::WEST
                }
            }
            // Greater Than
            '`' => stack_op!(self; x, y; if y > x { 1 } else { 0 }),
            // Hexadecimal Literals
            'a'..='f' => stack_op!(self; ; c.to_digit(16).unwrap() as i32),
            // Get
            'g' => stack_op!(self; y, x; grid.char_at(FungeVector(x, y))),
            // 'h' { Trefunge: Go High }
            // Input File
            'i' => {
                let filename: String = self.pop_t();
                let flags = self.pop();
                let pos: FungeVector = self.pop_t();
                if !Path::new(&filename).exists() {
                    self.delta.invert()
                } else {
                    let text = read_to_string(filename).unwrap_or_default();
                    grid.place(text, pos, flags & 1 != 0);
                }
            }
            // Jump Forward
            'j' => {
                let n = self.pop();
                if n < 0 {
                    for _ in 0..n.abs() {
                        self.walk_reverse(grid)
                    }
                } else {
                    for _ in 0..n {
                        self.walk(grid)
                    }
                }
            }
            // Iterate
            'k' => {
                let n = self.pop();
                if n == 0 {
                    return self.walk(grid);
                }
                let c = grid.runnable_char_ahead(self.pos, self.delta);
                for _ in 0..n {
                    self.command(c, grid, sender.clone(), out, quiet)
                }
            }
            // Lehmer Code Permutation
            'l' => {
                let n = self.pop();
                self.stacks[0].permute(n as usize);
            }
            // 'm' { Trefunge: High-Low If }
            // Clear Stack
            'n' => self.stacks[0].clear(),
            // Output File
            'o' => {
                let filename: String = self.pop_t();
                let path = Path::new(&filename);
                let flags = self.pop();
                let v_a: FungeVector = self.pop_t();
                let v_b: FungeVector = self.pop_t();
                let mut text = grid.read_from(v_a, v_b);
                if flags & 1 != 0 {
                    text = text
                        .lines()
                        .map(|l| l.strip_suffix("\r\n").or(l.strip_suffix("\n")).unwrap_or(l))
                        .collect();
                    text = text.trim_end().to_string();
                }
                if path.exists() && !path.metadata().unwrap().permissions().readonly() {
                    File::open(path)
                        .unwrap()
                        .write_all(text.as_bytes())
                        .unwrap();
                } else if !path.exists() {
                    File::create(path)
                        .unwrap()
                        .write_all(text.as_bytes())
                        .unwrap();
                } else {
                    self.delta.invert();
                }
            }
            // Put
            'p' => {
                let pos: FungeVector = self.pop_t();
                let c: char = self.pop_t();
                grid.set_char(pos + self.offset, c);
            }
            // Quit
            'q' => {
                let code = self.pop();
                sender.send(Event::Kill(code)).unwrap()
            }
            // Reflect
            'r' => self.delta.invert(),
            // Store Character
            's' => {
                let c: char = self.pop_t();
                let pos = grid.cell_ahead_ip(self);
                grid.set_char(pos, c);
                self.walk(grid);
            }
            // Split
            't' => sender.send(Event::Spawn(self.id)).unwrap(),
            // Stack under Stack
            'u' => {
                if self.stacks.len() == 1 {
                    return self.delta.invert();
                }
                let count = self.pop();
                if count > 0 {
                    for _ in 0..count {
                        let elem = self.stacks[1].pop();
                        self.push(elem);
                    }
                } else if count < 0 {
                    for _ in 0..count.abs() {
                        let elem = self.pop();
                        self.stacks[1].push(elem);
                    }
                }
            }
            // Go South
            'v' => self.delta = directions::SOUTH,
            // Compare
            'w' => {
                let (b, a) = (self.pop(), self.pop());
                if a < b {
                    self.delta.turn_left()
                } else if a > b {
                    self.delta.turn_right()
                };
            }
            // Absolute Delta
            'x' => self.delta = self.pop_t(),
            // Get SysInfo
            'y' => {
                let n = self.pop();
                let info: Vec<Box<fn(&FungeGrid, &mut InstructionPointer)>> = vec![
                    // 1: flags: getch, =, o, i, t
                    Box::new(|_, ip| ip.push(0b11111)),
                    // 2: bytes per cell
                    Box::new(|_, ip| ip.push(std::mem::size_of::<i32>() as i32)),
                    // 3: handprint           R  F  N  G
                    Box::new(|_, ip| ip.push(0x52_46_4E_47)),
                    // 4: version number
                    Box::new(|_, ip| {
                        ip.push(
                            env!("CARGO_PKG_VERSION")
                                .replace(".", "")
                                .parse::<i32>()
                                .unwrap(),
                        )
                    }),
                    // 5: how does "=" work
                    Box::new(|_, ip| ip.push(1)),
                    // 6: path separator
                    Box::new(|_, ip| ip.push(std::path::MAIN_SEPARATOR)),
                    // 7: dimension
                    Box::new(|_, ip| ip.push(2)),
                    // 8: pointer id
                    Box::new(|_, ip| ip.push(ip.id as i32)),
                    // 9: team number
                    Box::new(|_, ip| ip.push(0)),
                    // 10: pos
                    Box::new(|_, ip| ip.push(ip.pos)),
                    // 11: delta
                    Box::new(|_, ip| ip.push(ip.delta)),
                    // 12: storage offset
                    Box::new(|_, ip| ip.push(ip.offset)),
                    // 13: least point
                    Box::new(|_, ip| ip.push(directions::ORIGIN)),
                    // 14: greatest point
                    Box::new(|g, ip| ip.push(FungeVector(g.width() as i32, g.height() as i32 + 1))),
                    // 15: ((year - 1900) * 256 * 256) + (month * 256) + (day of month)
                    Box::new(|_, ip| {
                        let now = chrono::Utc::now();
                        ip.push(
                            ((now.year() - 1900) * 256 * 256)
                                + (now.month() as i32 * 256)
                                + now.day() as i32,
                        )
                    }),
                    // 16: (hour * 256 * 256) + (minute * 256) + (second)
                    Box::new(|_, ip| {
                        let now = chrono::Utc::now();
                        ip.push(
                            now.hour() as i32 * 256 * 256
                                + now.minute() as i32 * 256
                                + now.second() as i32,
                        )
                    }),
                    // 17: size of stack-stack
                    Box::new(|_, ip| ip.push(ip.stacks.len() as i32)),
                    // 18: size of stack
                    Box::new(|_, ip| {
                        let stack_lens = ip
                            .stacks
                            .iter()
                            .map(|s| s.len() as i32)
                            .collect::<Vec<i32>>();
                        for len in stack_lens {
                            ip.push(len);
                        }
                    }),
                    // 19: program arguments as 0gnirts, with another nul at end
                    Box::new(|_, ip| {
                        ip.push(args().collect::<Vec<String>>().join("\x00") + "\x00\x00")
                    }),
                    // 20: env vars as key=val 0nigrts, with another null at end
                    Box::new(|_, ip| {
                        ip.push(
                            vars()
                                .map(|(k, v)| format!("{k}={v}"))
                                .collect::<Vec<String>>()
                                .join("\x00")
                                + "\x00\x00",
                        )
                    }),
                ];
                match n {
                    ..=0 => info.iter().rev().for_each(|i| i(grid, self)),
                    1..=20 => info[n as usize - 1](grid, self),
                    21.. => (0..n - 20).for_each(|_| {
                        self.pop();
                    }),
                }
            }
            // No-Op
            'z' => {}
            // Begin Block
            '{' => {
                let n = self.pop();
                self.stacks.push_front(FungeStack::default());
                match n.signum() {
                    1 => {
                        let elems: Vec<i32> = (0..n).map(|_| self.stacks[1].pop()).collect();
                        for val in elems.iter().rev() {
                            self.push(*val)
                        }
                    }
                    -1 => {
                        for _ in 0..n.abs() {
                            self.stacks[1].push(0)
                        }
                    }
                    _ => {}
                }
                self.stacks[1].push(self.offset.0);
                self.stacks[1].push(self.offset.1);
                self.offset = self.pos + self.delta;
            }
            // North-South If
            '|' => {
                if self.pop() == 0 {
                    self.delta = directions::SOUTH
                } else {
                    self.delta = directions::NORTH
                }
            }
            // End Block
            '}' => {
                if self.stacks.len() == 1 {
                    return self.delta.invert();
                }
                let n = self.pop();
                (self.offset.1, self.offset.0) = (self.stacks[1].pop(), self.stacks[1].pop());
                if n > 0 {
                    let elems: Vec<i32> = (0..n).map(|_| self.pop()).collect();
                    for val in elems.iter().rev() {
                        self.stacks[1].push(*val)
                    }
                } else if n < 0 {
                    for _ in 0..n.abs() {
                        self.stacks[1].pop();
                    }
                }
                self.stacks.pop_front();
            }
            // Input Character
            '~' => sender
                .send(Event::Input(InputType::Character, self.id))
                .unwrap(),
            _ => self.delta.invert(),
        }
    }
}
