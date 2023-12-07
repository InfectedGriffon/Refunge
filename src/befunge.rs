use crate::arguments::Arguments;
use crate::event::{Event, EventHandler, KeyHandler, TickHandler};
use crate::grid::FungeGrid;
use crate::pointer::InstructionPointer;
use crate::{key, vector};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction::Horizontal, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use std::collections::VecDeque;
use std::fmt::Display;
use std::fs::read_to_string;
use std::io;
use std::str::FromStr;
use tui_textarea::TextArea;

#[derive(Default)]
pub struct Befunge<'a> {
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
    /// input for tui
    textarea: TextArea<'a>,
    inputting: bool,
    valid_input: bool,
    input_type: InputType,
    input_target: usize,

    /// exit code for q command
    pub exit_code: Option<i32>,
    /// stored command line arguments
    args: Arguments,
    /// global events
    events: EventHandler,
    /// tickspeed handling
    ticks: TickHandler,
    /// key input
    key_events: KeyHandler,
}
impl<'a> Befunge<'a> {
    /// create a new befunge simulation
    pub fn new(args: Arguments) -> Befunge<'a> {
        let paused = args.paused;
        let grid = FungeGrid::new(read_to_string(&args.file).expect("failed to read file"));
        let ip_list = [InstructionPointer::new(
            grid.start_pos(args.script),
            vector::EAST,
            0,
        )]
        .into();
        let mut textarea = TextArea::default();
        textarea.set_cursor_style(Style::default());
        Befunge {
            grid,
            ip_list,
            paused,
            textarea,
            args,
            ..Default::default()
        }
    }
    /// step forward once and run whatever char we're standing on
    pub fn tick(&mut self) {
        for ip in self.ip_list.iter_mut() {
            if ip.dead {
                continue;
            }
            if !ip.first_tick {
                ip.walk(&self.grid)
            }
            let c = self.grid.char_at(ip.pos);
            if ip.string_mode {
                match c {
                    '"' => ip.string_mode = false,
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
                ip.command(
                    c,
                    &mut self.grid,
                    self.events.sender.clone(),
                    &mut self.out,
                    self.args.quiet,
                );
            }
            if ip.first_tick {
                ip.first_tick = false
            }
        }
        if let Some(event) = self.events.next() {
            match event {
                Event::Spawn(id) => {
                    let mut new_ip = self.ip_list[id].clone();
                    new_ip.delta.invert();
                    self.ip_list.insert(id, new_ip);
                    for (idx, ip) in self.ip_list.iter_mut().enumerate() {
                        ip.id = idx
                    }
                }
                Event::Kill(code) => {
                    self.exit_code = Some(code);
                    for ip in self.ip_list.iter_mut() {
                        ip.dead = true
                    }
                }
                Event::Input(t, id) => {
                    if self.args.quiet {
                        self.ip_list[id].push(t.parse_stdin());
                    } else {
                        self.inputting = true;
                        self.input_type = t;
                        self.input_target = id;
                        let title = match t {
                            InputType::Number => "Input Number",
                            InputType::Character => "Input Character",
                        };
                        self.textarea
                            .set_block(Block::default().borders(Borders::ALL).title(title));
                    }
                }
            }
        }
    }
    /// reset everything
    pub fn restart(&mut self) {
        self.grid.reset();
        self.ip_list = [InstructionPointer::new(
            self.grid.start_pos(self.args.script),
            vector::EAST,
            0,
        )]
        .into();
        self.out.clear();
        self.paused = self.args.paused;
        self.textarea = TextArea::default();
        self.textarea.set_cursor_style(Style::default());
    }

    /// is there a tick available
    pub fn has_tick(&self) -> bool {
        self.ticks.has_tick()
    }
    /// handle key input for scrolling, pausing, etc
    pub fn handle_key_events(&mut self) -> bool {
        if let Some(event) = self.key_events.next() {
            if matches!(event, key!(ctrl;'c')) {
                return true;
            } // give priority to input events
            if self.inputting {
                self.handle_tui_input(event);
                return false;
            }
            match event {
                key!('.') => self.ticks.speed_up(),
                key!(',') => self.ticks.slow_down(),
                key!(Right) if self.paused => self.tick(),
                key!('p') => self.paused = !self.paused,
                key!('h') => self.grid_scroll.1 = self.grid_scroll.1.saturating_sub(1),
                key!('j') => self.grid_scroll.0 += 1,
                key!('k') => self.grid_scroll.0 = self.grid_scroll.0.saturating_sub(1),
                key!('l') => self.grid_scroll.1 += 1,
                key!('i') => self.output_scroll = self.output_scroll.saturating_sub(1),
                key!('o') => self.output_scroll += 1,
                key!('r') => self.restart(),
                key!('q') if self.ended() => return true,
                _ => {}
            }
        }
        false
    }
    fn handle_tui_input(&mut self, event: KeyEvent) {
        if matches!(event, key!(Enter)) {
            if self.valid_input {
                let text = self.textarea.lines().last().unwrap();
                self.ip_list[self.input_target].push(self.input_type.parse(text));
                self.inputting = false;
            }
            self.textarea.move_cursor(tui_textarea::CursorMove::Head);
            self.textarea.delete_line_by_end();
            return;
        }
        if self.textarea.input(event) {
            if self.input_type.can_parse(&self.textarea.lines()[0]) {
                self.textarea
                    .set_style(Style::default().fg(Color::LightGreen));
                self.valid_input = true;
            } else {
                self.textarea
                    .set_style(Style::default().fg(Color::LightRed));
                self.valid_input = false;
            }
        }
    }
    /// is the tui paused
    pub fn paused(&self) -> bool {
        self.paused || self.inputting
    }
    /// has the interpreter reached the end
    pub fn ended(&self) -> bool {
        self.ip_list.iter().all(|ip| ip.dead)
    }

    /// log the contents of all IPs' stacks
    pub fn log_stacks(&self) {
        println!("Final stack contents:");
        for (idx, ip) in self.ip_list.iter().enumerate() {
            println!("IP {idx}: {:?}", ip.stacks);
        }
    }
    fn stack_constraints(&self) -> Vec<Constraint> {
        let mut arr = vec![];
        for ip in &self.ip_list {
            arr.push(Constraint::Length(1));
            for _ in &ip.stacks {
                arr.push(Constraint::Length(8));
            }
        }
        arr.push(Constraint::Min(1));
        arr
    }
    fn max_stack_len(&self) -> u16 {
        self.ip_list
            .iter()
            .map(|ip| ip.stacks.iter().max_by_key(|s| s.len()).unwrap().len())
            .max()
            .unwrap() as u16
    }

    /// render the grid, stack, output, and message
    pub fn render(&mut self, f: &mut Frame) {
        let grid_width = (self.grid.width() as u16 + 2).clamp(20, 80);
        let grid_height = (self.grid.height() as u16 + 2).clamp(9, 25);
        let output_height = textwrap::wrap(&self.out, grid_width as usize - 2).len() as u16 + 2;
        let stack_height = (grid_height + output_height).max(self.max_stack_len() + 2);
        let chunks = Layout::new()
            .constraints(vec![Constraint::Length(grid_width), Constraint::Min(0)])
            .direction(Horizontal)
            .split(f.size());
        let column_a = Layout::new()
            .constraints(vec![
                Constraint::Length(grid_height),
                Constraint::Length(output_height),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(chunks[0]);
        let column_b = Layout::new()
            .constraints([Constraint::Length(stack_height), Constraint::Min(1)])
            .split(chunks[1]);
        let stack_zone = Layout::new()
            .constraints(self.stack_constraints())
            .direction(Horizontal)
            .split(column_b[0]);
        let output = Paragraph::new(self.out.clone())
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Output"))
            .scroll((self.output_scroll, 0));

        f.render_widget(
            self.grid.clone().highlights(self.ip_list.clone()),
            column_a[0],
        );
        f.render_widget(output, column_a[1]);
        if self.inputting {
            f.render_widget(self.textarea.widget(), column_a[2])
        }
        if self.ended() {
            f.render_widget(
                Paragraph::new("Funge ended.\nPress r to restart,\nor q to exit."),
                column_a[2],
            )
        }
        let mut index = 0;
        for ip in &self.ip_list {
            f.render_widget(
                Paragraph::new(format!("IP {}", ip.id))
                    .wrap(Wrap { trim: true })
                    .block(Block::default().borders(Borders::TOP | Borders::BOTTOM)),
                stack_zone[index],
            );
            index += 1;
            for stack in &ip.stacks {
                stack.render(f, stack_zone[index], stack_height, "Stack");
                index += 1;
            }
        }
        if self.paused {
            f.render_widget(Paragraph::new("paused"), column_b[1])
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub enum InputType {
    #[default]
    Number,
    Character,
}
impl InputType {
    /// parse some text into the desired type
    fn parse(&self, text: &str) -> i32 {
        match self {
            InputType::Number => text.parse().unwrap_or_default(),
            InputType::Character => text.parse::<char>().unwrap_or_default() as i32,
        }
    }
    /// parse user input into the desired type
    fn parse_stdin(&self) -> i32 {
        match self {
            InputType::Number => parse_input(),
            InputType::Character => parse_input::<char>() as i32,
        }
    }
    /// check if a string would be valid if it was parsed as the desired type
    fn can_parse(&self, text: &str) -> bool {
        match self {
            InputType::Number => text.parse::<i32>().is_ok(),
            InputType::Character => text.parse::<char>().is_ok(),
        }
    }
}

/// loop getting inputs until the user enters one that can be parsed
fn parse_input<T>() -> T
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    let mut buffer = String::new();
    loop {
        buffer.clear();
        io::stdin().read_line(&mut buffer).unwrap();
        match buffer.trim().parse() {
            Ok(parsed) => return parsed,
            Err(err) => eprintln!("\x1b[31m{err}\x1b[m"),
        }
    }
}
