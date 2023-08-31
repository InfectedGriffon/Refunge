use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use crossterm::event::{KeyEvent, Event as CrosstermEvent, poll, read};
use anyhow::Result;

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
}

/// provides events and handles changing of tickrate
pub struct EventHandler {
    tickrate: Arc<Mutex<Duration>>,
    receiver: mpsc::Receiver<Event>
}
impl EventHandler {
    /// creates a new event handler with a default tickrate of 128
    pub fn new() -> EventHandler {
        let (sender, receiver) = mpsc::channel::<Event>();
        let inner_tickrate = Arc::new(Mutex::new(Duration::from_millis(128)));
        let tickrate = Arc::clone(&inner_tickrate);

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = inner_tickrate.lock().unwrap().saturating_sub(last_tick.elapsed());
                if poll(timeout).unwrap_or(false) {
                    if let CrosstermEvent::Key(key) = read().unwrap() {
                        sender.send(Event::Key(key)).unwrap();
                    }
                }
                if last_tick.elapsed() >= *inner_tickrate.lock().unwrap() {
                    sender.send(Event::Tick).unwrap_or(());
                    last_tick = Instant::now();
                }
            }
        });

        EventHandler { tickrate, receiver }
    }

    /// next event in the queue
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
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
impl Default for EventHandler {
    fn default() -> EventHandler {
        EventHandler::new()
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
