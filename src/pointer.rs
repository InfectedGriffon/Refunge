use std::env::{args, vars};
use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;
use chrono::{Datelike, Timelike};
use crate::event::Event;
use crate::grid::FungeGrid;
use crate::input::take_input_parse;
use crate::stack::FungeStack;
use crate::vector;
use crate::vector::FungeVector;

/// a befunge ip, with a 2d coordinate and direction
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
    /// create a new instruction pointer facing right at specified coordinates
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
    /// push a 2d coordinate onto the stack
    pub fn push_vector(&mut self, pos: FungeVector) {
        self.push(pos.0);
        self.push(pos.1);
    }

    pub fn command(
        &mut self,
        c: char,
        grid: &mut FungeGrid,
        sender: mpsc::Sender<Event>,
        out: &mut String,
        quiet: bool
    ) {
        match c {
            ' ' => {
                while grid.char_at(self.pos) == ' ' { self.walk(grid) }
                self.walk_reverse(grid);
            }
            '!' => {
                let n = self.pop();
                self.push(if n == 0 {1} else {0})
            }
            '"' => self.string_mode = true,
            '#' => self.walk(grid),
            '$' => {self.pop();}
            '%' => {
                let (x, y) = (self.pop(), self.pop());
                if x == 0 {self.push(0)} else {self.push(y % x)}
            }
            '&' => self.push(take_input_parse::<i32>("enter a number").unwrap()),
            '\'' => {
                let c = grid.char_at(grid.cell_ahead_ip(self));
                self.push(c as i32);
                self.walk(grid);
            }
            // Fingerprints: ()
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
                if quiet {
                    print!("{c}");
                } else {
                    out.push(c);
                }
            }
            '-' => {
                let (x, y) = (self.pop(), self.pop());
                self.push(y.saturating_sub(x));
            }
            '.' => {
                let n = self.pop();
                if quiet {
                    print!("{n} ");
                } else {
                    out.push_str(&format!("{n} "));
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
                self.walk(grid); // move off of current semicolon
                while grid.char_at(self.pos) != ';' {self.walk(grid)}
            },
            '<' => self.delta = vector::WEST,
            '=' => {
                let cmd = self.pop_0gnirts();
                self.push(Command::new("cmd.exe")
                    .args(vec!["/c", &cmd])
                    .status()
                    .expect("failed to execute")
                    .code()
                    .unwrap_or_default());
            }
            '>' => self.delta = vector::EAST,
            '?' => self.delta = rand::random(),
            '@' => self.dead = true,
            // Fingerprints: A-Z
            '[' => self.delta.turn_left(),
            '\\' => { let xy = FungeVector(self.pop(), self.pop()); self.push_vector(xy) }
            ']' => self.delta.turn_right(),
            '^' => self.delta = vector::NORTH,
            '_' => if self.pop() == 0 {self.delta = vector::EAST } else {self.delta = vector::WEST },
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
                let c = grid.char_at(FungeVector(x, y));
                self.push(c as i32)
            },
            // trefunge h
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
            'j' => {
                let n = self.pop();
                if n < 0 {
                    for _ in 0..n.abs() {self.walk_reverse(grid)}
                } else if n > 0 {
                    for _ in 0..n {self.walk(grid)}
                }
            }
            'k' => {
                let n = self.pop();
                if n == 0 {return self.walk(grid)}
                let c = grid.runnable_char_ahead(self.pos, self.delta);
                for _ in 0..n { self.command(c, grid, sender.clone(), out, quiet) }
            }
            'l' => {
                let n = self.pop();
                self.stacks[0].permute(n as usize);
            }
            // trefunge m
            'n' => self.stacks[0].clear(),
            'o' => {
                let filename = self.pop_0gnirts();
                let path = Path::new(&filename);
                let flags = self.pop();
                let v_a = self.pop_vector();
                let v_b = self.pop_vector();
                let mut text = grid.read_from(v_a, v_b);
                if flags & 1 != 0 {
                    text = text.lines().map(|l| l.strip_suffix("\r\n").or(l.strip_suffix("\n")).unwrap_or(l)).collect();
                    text = text.trim_end().to_string();
                }
                if path.exists() && !path.metadata().unwrap().permissions().readonly() {
                    File::open(path).unwrap().write_all(text.as_bytes()).unwrap();
                } else if !path.exists() {
                    File::create(path).unwrap().write_all(text.as_bytes()).unwrap();
                } else {
                    self.delta.invert();
                }
            }
            'p' => {
                let pos = self.pop_vector();
                let c = self.pop_char();
                grid.set_char(pos + self.offset, c);
            },
            'q' => sender.send(Event::Kill).unwrap(),
            'r' => self.delta.invert(),
            's' => {
                let c = self.pop_char();
                let pos = grid.cell_ahead_ip(self);
                grid.set_char(pos, c);
                self.walk(grid);
            },
            't' => sender.send(Event::Spawn(self.id)).unwrap(),
            'u' => {
                if self.stacks.len() == 1 { return self.delta.invert() }
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
            'v' => self.delta = vector::SOUTH,
            'w' => {
                let (b, a) = (self.pop(), self.pop());
                if a < b {
                    self.delta.turn_left()
                } else if a > b {
                    self.delta.turn_right()
                };
            }
            'x' => self.delta = self.pop_vector(),
            'y' => {
                let n = self.pop();
                let info: Vec<Box<fn(&FungeGrid, &mut InstructionPointer)>> = vec![
                    // 1: flags: getch, =, o, i, t
                    Box::new(|_,ip| ip.push(0b11111)),
                    // 2: bytes per cell
                    Box::new(|_,ip| ip.push(std::mem::size_of::<i32>() as i32)),
                    // 3: handprint           R  F  N  G
                    Box::new(|_,ip| ip.push(0x52_46_4E_47)),
                    // 4: version number
                    Box::new(|_,ip| ip.push(env!("CARGO_PKG_VERSION").replace(".","").parse().unwrap())),
                    // 5: how does "=" work
                    Box::new(|_,ip| ip.push(1)),
                    // 6: path separator
                    Box::new(|_,ip| ip.push(std::path::MAIN_SEPARATOR as i32)),
                    // 7: dimension
                    Box::new(|_,ip| ip.push(2)),
                    // 8: pointer id
                    Box::new(|_,ip| ip.push(ip.id as i32)),
                    // 9: team number
                    Box::new(|_,ip| ip.push(0)),
                    // 10: pos
                    Box::new(|_,ip| ip.push_vector(ip.pos)),
                    // 11: delta
                    Box::new(|_,ip| ip.push_vector(ip.delta)),
                    // 12: storage offset
                    Box::new(|_,ip| ip.push_vector(ip.offset)),
                    // 13: least point
                    Box::new(|_,ip| ip.push_vector(vector::ORIGIN)),
                    // 14: greatest point
                    Box::new(|g,ip| ip.push_vector(FungeVector(g.width() as i32, g.height() as i32+1))),
                    // 15: ((year - 1900) * 256 * 256) + (month * 256) + (day of month)
                    Box::new(|_,ip| { let now = chrono::Utc::now(); ip.push(((now.year()-1900)*256*256) + (now.month() as i32*256) + now.day() as i32) }),
                    // 16: (hour * 256 * 256) + (minute * 256) + (second)
                    Box::new(|_,ip| { let now = chrono::Utc::now(); ip.push(now.hour() as i32*256*256 + now.minute() as i32*256 + now.second() as i32) }),
                    // 17: size of stack-stack
                    Box::new(|_,ip| ip.push(ip.stacks.len() as i32)),
                    // 18: size of stack
                    Box::new(|_,ip| ip.push(ip.stacks[0].len() as i32)),
                    // 19: program arguments as 0gnirts, with another nul at end
                    Box::new(|_,ip| ip.push_0gnirts(args().collect::<Vec<String>>().join("\x00") + "\x00\x00")),
                    // 20: env vars as key=val 0nigrts, with another null at end
                    Box::new(|_,ip| ip.push_0gnirts(vars().map(|(k,v)|format!("{k}={v}")).collect::<Vec<String>>().join("\x00") + "\x00\x00")),
                ];
                match n {
                    ..=0 => info.iter().rev().for_each(|i| i(grid, self)),
                    1..=20 => info[n as usize-1](grid, self),
                    21.. => (0..n-20).for_each(|_|{self.pop();})
                }
            },
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
                self.stacks[1].push(self.offset.0);
                self.stacks[1].push(self.offset.1);
                self.offset = self.pos + self.delta;
            }
            '|' => if self.pop() == 0 {self.delta = vector::SOUTH} else {self.delta = vector::NORTH},
            '}' => {
                if self.stacks.len() == 1 { return self.delta.invert() }
                let n = self.pop();
                (self.offset.1, self.offset.0) = (self.stacks[1].pop(), self.stacks[1].pop());
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
            '~' => self.push(take_input_parse::<char>("enter a character").unwrap() as i32),
            _ => self.delta.invert(),
        }
    }
}
