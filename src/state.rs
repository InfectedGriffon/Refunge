use ratatui::widgets::Paragraph;
use crate::state::{EndAt::*, OnTick::*};

#[derive(PartialEq, Debug)]
pub struct FungeState {
    /// when to end this state
    ends_at: EndAt,
    /// whether the Program Counter moves during this state
    moving: bool,
    /// whether to run instructions, push characters, etc
    action: OnTick,
    /// how many ticks this state has lasted for
    ticks: u32,
    /// printed under the output area
    message: &'static str,
}
impl FungeState {
    /// generates a new state with message
    const fn of_message(ends_at: EndAt, moving: bool, action: OnTick, message: &'static str) -> FungeState {
        FungeState { ends_at, moving, action, ticks: 0, message }
    }
    /// generates a new state without a message
    const fn of(ends_at: EndAt, moving: bool, action: OnTick) -> FungeState {
        FungeState { ends_at, moving, action, ticks: 0, message: "" }
    }
    /// whether the Program Counter moves during this state
    pub fn moving(&self) -> bool {self.moving}
    /// whether to run instructions, push characters, etc
    pub fn action(&self) -> OnTick {self.action}
    /// increment this state's tick count
    pub fn tick(&mut self) {self.ticks += 1}
    /// display message as a paragraph
    pub fn render_message(&self, input: &str) -> Paragraph {
        if self.inputting_num() || self.inputting_char() {
            Paragraph::new(format!("{} {}", self.message, input))
        } else {
            Paragraph::new(self.message)
        }
    }
    /// has this state ended yet?
    pub fn is_over(&self, c: char) -> bool {
        match self.ends_at {
            Instant => true,
            Ticks(n) => self.ticks == n as u32,
            Char(target) => self.ticks > 0 && c == target,
            Manual | Never => false,
        }
    }
    /// is this state the end of the program
    pub fn is_end(&self) -> bool {self.moving == false && self.action == Nothing}
    /// are we inputting numbers
    pub fn inputting_num(&self) -> bool {self.message == "input num:"}
    /// are we inputting characters
    pub fn inputting_char(&self) -> bool {self.message == "input char:"}
}
impl Default for FungeState { fn default() -> Self { STARTED } }

#[derive(PartialEq, Debug, Default)]
enum EndAt {
    /// single action
    #[default]
    Instant,
    /// last some amount of time
    Ticks(u8),
    /// last until reaching another character
    Char(char),
    /// deactivated by some other event
    Manual,
    /// can still be changed by other state-setters
    Never
}
#[derive(PartialEq, Copy, Clone, Debug, Default)]
pub enum OnTick {
    /// nop
    Nothing,
    /// run commands like normal
    #[default]
    Instruction,
    /// push characters to stack
    StringPush
}

pub const STARTED: FungeState        = FungeState::of(Instant, false, Instruction);
pub const RUNNING: FungeState        = FungeState::of(Never, true, Instruction);
pub const ENDED: FungeState          = FungeState::of_message(Never, false, Nothing, "sim ended.\npress r to restart or q to exit.");
pub const SKIP_NEXT: FungeState      = FungeState::of(Ticks(1), true, Nothing);
pub const SKIP_UNTIL: FungeState     = FungeState::of(Char(';'), true, Nothing);
pub const STRING_MODE: FungeState    = FungeState::of_message(Char('"'), true, StringPush, "(string mode)");
pub const INPUTTING_CHAR: FungeState = FungeState::of_message(Manual, false, Nothing, "input char:");
pub const INPUTTING_NUM: FungeState  = FungeState::of_message(Manual, false, Nothing, "input num:");
