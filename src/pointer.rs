use crate::befunge::InputType;
use crate::event::Event;
use crate::grid::FungeGrid;
use crate::stack::FungeStack;
use crate::vector;
use crate::vector::FungeVector;
use chrono::{Datelike, Timelike};
use std::env::{args, vars};
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;

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
    pub fn walk_reverse(&mut self, grid: &FungeGrid) {
        self.pos.0 = (self.pos.0 - self.delta.0).rem_euclid(grid.width() as i32);
        self.pos.1 = (self.pos.1 - self.delta.1).rem_euclid(grid.height() as i32);
    }

    /// get the top value from the stack
    pub fn pop(&mut self) -> i32 {
        self.stacks[0].pop()
    }
    /// get the top value from the stack as a character
    pub fn pop_char(&mut self) -> char {
        char::from_u32(self.pop() as u32).unwrap_or(' ')
    }
    /// pop values from the stack and parse into chars until a zero is found
    pub fn pop_0gnirts(&mut self) -> String {
        let mut output = String::new();
        loop {
            let c = self.pop();
            if c == 0 {
                return output;
            }
            output.push(char::from_u32(c as u32).unwrap_or(' '));
        }
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
    /// push a 2d coordinate onto the stack
    pub fn push_vector(&mut self, pos: FungeVector) {
        self.push(pos.0);
        self.push(pos.1);
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
                self.walk_reverse(grid)
            }
            // Logical Not
            '!' => {
                let n = self.pop();
                self.push(if n == 0 { 1 } else { 0 })
            }
            // Toggle Stringmode
            '"' => self.string_mode = true,
            // Trampoline
            '#' => self.walk(grid),
            // Pop
            '$' => {
                self.pop();
            }
            // Remainder
            '%' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(y.checked_rem(x).unwrap_or_default());
            }
            // Input Integer
            '&' => sender
                .send(Event::Input(InputType::Number, self.id))
                .unwrap(),
            // Fetch Character
            '\'' => {
                self.walk(grid);
                self.push(grid.char_at(self.pos) as i32);
            }
            // '(' { Fingerprints: Load Semantics }
            // ')' { Fingerprints: Unload Semantics }
            // Multiply
            '*' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(x.saturating_mul(y))
            }
            // Add
            '+' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(x.saturating_add(y))
            }
            // Output Character
            ',' => {
                let c = self.pop_char();
                if quiet {
                    print!("{c}");
                } else {
                    out.push(c);
                }
            }
            // Subtract
            '-' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(y.saturating_sub(x));
            }
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
            '/' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(y.checked_div(x).unwrap_or_default());
            }
            // Decimal Literals
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
            // Duplicate
            ':' => {
                let n = self.pop();
                self.push(n);
                self.push(n);
            }
            // Jump Over
            ';' => {
                self.walk(grid); // move off of current semicolon
                while grid.char_at(self.pos) != ';' {
                    self.walk(grid);
                }
            }
            // Go West
            '<' => self.delta = vector::WEST,
            // Execute
            '=' => {
                let cmd = self.pop_0gnirts();
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
            '>' => self.delta = vector::EAST,
            // Go Away
            '?' => self.delta = rand::random(),
            // Stop
            '@' => self.dead = true,
            // 'A'...'Z' { Fingerprints }
            // Turn Left
            '[' => self.delta.turn_left(),
            // Swap
            '\\' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(x);
                self.push(y);
            }
            // Turn Right
            ']' => self.delta.turn_right(),
            // Go North
            '^' => self.delta = vector::NORTH,
            // East-West If
            '_' => {
                if self.pop() == 0 {
                    self.delta = vector::EAST
                } else {
                    self.delta = vector::WEST
                }
            }
            // Greater Than
            '`' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(if y > x { 1 } else { 0 })
            }
            // Hexadecimal Literals
            'a' => self.push(10),
            'b' => self.push(11),
            'c' => self.push(12),
            'd' => self.push(13),
            'e' => self.push(14),
            'f' => self.push(15),
            // Get
            'g' => {
                let (y, x) = (self.pop(), self.pop());
                let c = grid.char_at(FungeVector(x, y));
                self.push(c as i32)
            }
            // 'h' { Trefunge: Go High }
            // Input File
            'i' => {
                let filename = self.pop_0gnirts();
                let flags = self.pop();
                let pos = self.pop_vector();
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
                let filename = self.pop_0gnirts();
                let path = Path::new(&filename);
                let flags = self.pop();
                let v_a = self.pop_vector();
                let v_b = self.pop_vector();
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
                let pos = self.pop_vector();
                let c = self.pop_char();
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
                let c = self.pop_char();
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
            // Go South
            'v' => self.delta = vector::SOUTH,
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
            'x' => self.delta = self.pop_vector(),
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
                        ip.push(env!("CARGO_PKG_VERSION").replace(".", "").parse().unwrap())
                    }),
                    // 5: how does "=" work
                    Box::new(|_, ip| ip.push(1)),
                    // 6: path separator
                    Box::new(|_, ip| ip.push(std::path::MAIN_SEPARATOR as i32)),
                    // 7: dimension
                    Box::new(|_, ip| ip.push(2)),
                    // 8: pointer id
                    Box::new(|_, ip| ip.push(ip.id as i32)),
                    // 9: team number
                    Box::new(|_, ip| ip.push(0)),
                    // 10: pos
                    Box::new(|_, ip| ip.push_vector(ip.pos)),
                    // 11: delta
                    Box::new(|_, ip| ip.push_vector(ip.delta)),
                    // 12: storage offset
                    Box::new(|_, ip| ip.push_vector(ip.offset)),
                    // 13: least point
                    Box::new(|_, ip| ip.push_vector(vector::ORIGIN)),
                    // 14: greatest point
                    Box::new(|g, ip| {
                        ip.push_vector(FungeVector(g.width() as i32, g.height() as i32 + 1))
                    }),
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
                        ip.push_0gnirts(args().collect::<Vec<String>>().join("\x00") + "\x00\x00")
                    }),
                    // 20: env vars as key=val 0nigrts, with another null at end
                    Box::new(|_, ip| {
                        ip.push_0gnirts(
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
                    self.delta = vector::SOUTH
                } else {
                    self.delta = vector::NORTH
                }
            }
            // End Block
            '}' => {
                if self.stacks.len() == 1 {
                    return self.delta.invert();
                }
                let n = self.pop();
                (self.offset.1, self.offset.0) = (self.stacks[1].pop(), self.stacks[1].pop());
                match n.signum() {
                    1 => {
                        let elems: Vec<i32> = (0..n).map(|_| self.pop()).collect();
                        for val in elems.iter().rev() {
                            self.stacks[1].push(*val)
                        }
                    }
                    -1 => {
                        for _ in 0..n.abs() {
                            self.stacks[1].pop();
                        }
                    }
                    _ => {}
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
