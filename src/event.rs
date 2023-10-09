use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use crossterm::event::{KeyEvent, Event as CrosstermEvent, poll, read};
use anyhow::Result;

#[derive(Clone, Debug)]
pub enum Event {
    Spawn(usize),
    Kill,
}

pub struct EventHandler {
    pub sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>
}
impl EventHandler {
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

pub struct TickHandler {
    tickrate: Arc<Mutex<Duration>>,
    receiver: mpsc::Receiver<()>
}
impl TickHandler {
    pub fn has_tick(&self) -> bool {
        self.receiver.try_recv().is_ok()
    }
     /// double the speed, up to a maximum of one tick per 16 milliseconds
    pub fn speed_up(&self) {
        let mut tickrate = self.tickrate.lock().unwrap();
        *tickrate = Duration::from_millis((tickrate.as_millis()/2).max(16) as u64);
    }
    /// half the speed, down to a minimum of one tick per about one second
    pub fn slow_down(&self) {
        let mut tickrate = self.tickrate.lock().unwrap();
        *tickrate = Duration::from_millis((tickrate.as_millis()*2).min(1024) as u64)
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

pub struct KeyHandler {
    receiver: mpsc::Receiver<KeyEvent>
}
impl KeyHandler {
    pub fn new() -> KeyHandler {
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
    pub fn next(&self) -> Result<KeyEvent> {
        Ok(self.receiver.try_recv()?)
    }
}
impl Default for KeyHandler {
    fn default() -> KeyHandler {
        KeyHandler::new()
    }
}

#[macro_export]
macro_rules! key {
    ($char:literal) => {
        KeyEvent{code:KeyCode::Char($char),modifiers:KeyModifiers::NONE,kind:KeyEventKind::Release,..}
    };
    ($key:ident) => {
        KeyEvent{code:KeyCode::$key,modifiers:KeyModifiers::NONE,kind:KeyEventKind::Release,..}
    };
    (ctrl;$char:literal) => {
        KeyEvent{code:KeyCode::Char($char),modifiers:KeyModifiers::CONTROL,kind:KeyEventKind::Release,..}
    };
    (ctrl;$key:ident) => {
        KeyEvent{code:KeyCode::$key,modifiers:KeyModifiers::CONTROL,kind:KeyEventKind::Release,..}
    }
}
