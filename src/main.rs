#[macro_use]
extern crate clap;
extern crate config;

use std::io;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

mod configs;
mod events;
mod model;
mod view;

fn main() -> Result<(), io::Error> {
    let config = configs::Config::new();

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = events::Events::new();
    let mut model = model::Model::new(config);

    loop {
        terminal.draw(|mut f| {
            view::draw(&mut model, &mut f);
        })?;

        match events.next().unwrap() {
            events::Event::Input(key) => match key {
                Key::Char('q') => break,
                _ => {
                    model.handle(key);
                }
            },
            _ => {}
        }

        model.tick();
    }

    Ok(())
}
