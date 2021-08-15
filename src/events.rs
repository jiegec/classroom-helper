use crossterm::event;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum Event {
    Input(event::KeyEvent),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event>,
    _input_handle: thread::JoinHandle<()>,
    _tick_handle: thread::JoinHandle<()>,
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = mpsc::channel();
        let input_handle = {
            let tx = tx.clone();
            thread::spawn(move || loop {
                match event::read() {
                    Ok(event::Event::Key(event)) => {
                        if let Err(_) = tx.send(Event::Input(event)) {
                            return;
                        }
                    }
                    _ => {}
                }
            })
        };
        let tick_handle = {
            let tx = tx.clone();
            thread::spawn(move || {
                let tx = tx.clone();
                loop {
                    tx.send(Event::Tick).unwrap();
                    thread::sleep(Duration::from_millis(100));
                }
            })
        };
        Events {
            rx,
            _input_handle: input_handle,
            _tick_handle: tick_handle,
        }
    }

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.rx.recv()
    }
}
