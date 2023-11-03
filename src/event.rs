use crate::befunge::InputType;
use crossterm::event::{poll, read, Event as CrosstermEvent, KeyEvent};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// global events used to communicate from IP to befunge
/// controls adding IPs, exiting, and inputting
#[derive(Clone, Debug)]
pub enum Event {
    /// spawn an IP based on one with given index
    Spawn(usize),
    /// stop the program with a given exit code
    Kill(i32),
    /// called from an IP with a given index
    /// will pause tui to allow for input
    Input(InputType, usize),
}

/// multi-producer, single-receiver channel for global events
pub struct EventHandler {
    /// clone this to make more inputs
    pub sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
}
impl EventHandler {
    /// returns the next event if it exists
    pub fn next(&self) -> Option<Event> {
        self.receiver.try_recv().ok()
    }
}
impl Default for EventHandler {
    fn default() -> EventHandler {
        let (sender, receiver) = mpsc::channel();
        EventHandler { sender, receiver }
    }
}

/// sends out a tick event based on the supplied tickrate
pub struct TickHandler {
    tickrate: Arc<Mutex<Duration>>,
    receiver: mpsc::Receiver<()>,
}
impl TickHandler {
    /// returns true if a tick has been produced since last called
    pub fn has_tick(&self) -> bool {
        self.receiver.try_recv().is_ok()
    }
    /// double the speed, up to a maximum of one tick per 16 milliseconds
    pub fn speed_up(&self) {
        let mut tickrate = self.tickrate.lock().unwrap();
        *tickrate = Duration::from_millis((tickrate.as_millis() / 2).max(16) as u64);
    }
    /// half the speed, down to a minimum of one tick per about one second
    pub fn slow_down(&self) {
        let mut tickrate = self.tickrate.lock().unwrap();
        *tickrate = Duration::from_millis((tickrate.as_millis() * 2).min(1024) as u64)
    }
}
impl Default for TickHandler {
    fn default() -> TickHandler {
        let (inner_sender, receiver) = mpsc::channel();
        let inner_tickrate = Arc::new(Mutex::new(Duration::from_millis(128)));
        let tickrate = Arc::clone(&inner_tickrate);
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                if last_tick.elapsed() >= *inner_tickrate.lock().unwrap() {
                    inner_sender.send(()).unwrap();
                    last_tick = Instant::now();
                }
            }
        });
        TickHandler { tickrate, receiver }
    }
}

/// wrapper around an infinitely looping thread waiting for key input
pub struct KeyHandler {
    receiver: mpsc::Receiver<KeyEvent>,
}
impl KeyHandler {
    /// returns the next key input if it exists
    pub fn next(&self) -> Option<KeyEvent> {
        self.receiver.try_recv().ok()
    }
}
impl Default for KeyHandler {
    fn default() -> KeyHandler {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || loop {
            if poll(Duration::ZERO).unwrap_or(false) {
                if let CrosstermEvent::Key(key) = read().unwrap() {
                    sender.send(key).unwrap_or(());
                }
            }
        });
        KeyHandler { receiver }
    }
}

#[macro_export]
macro_rules! key {
    ($char:literal) => {
        ::crossterm::event::KeyEvent {
            code: KeyCode::Char($char),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            ..
        }
    };
    ($key:ident) => {
        ::crossterm::event::KeyEvent {
            code: KeyCode::$key,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            ..
        }
    };
    (ctrl;$char:literal) => {
        ::crossterm::event::KeyEvent {
            code: KeyCode::Char($char),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Release,
            ..
        }
    };
    (ctrl;$key:ident) => {
        ::crossterm::event::KeyEvent {
            code: KeyCode::$key,
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Release,
            ..
        }
    };
}
