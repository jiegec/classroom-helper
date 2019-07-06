use crate::model::{Model, UiWidget};
use std::io;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::terminal::Frame;
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
use tui::Terminal;

pub fn draw<B: Backend>(model: &Model, mut f: &mut Frame<B>) {
    let highlighted_style = Style::default().fg(Color::Gray);
    let normal_style = Style::default();

    let chunks_virt = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let chunks_left = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(chunks_virt[0]);
    Block::default()
        .title("Students")
        .borders(Borders::ALL)
        .border_style(if let UiWidget::Student = model.current {
            highlighted_style
        } else {
            normal_style
        })
        .title_style(if let UiWidget::Student = model.current {
            highlighted_style
        } else {
            normal_style
        })
        .render(&mut f, chunks_left[0]);
    Block::default()
        .title("Status")
        .borders(Borders::ALL)
        .border_style(if let UiWidget::Status = model.current {
            highlighted_style
        } else {
            normal_style
        })
        .title_style(if let UiWidget::Status = model.current {
            highlighted_style
        } else {
            normal_style
        })
        .render(&mut f, chunks_left[1]);

    let chunks_right = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(40),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(chunks_virt[1]);
    Block::default()
        .title("Log")
        .borders(Borders::ALL)
        .border_style(if let UiWidget::Log = model.current {
            highlighted_style
        } else {
            normal_style
        })
        .title_style(if let UiWidget::Log = model.current {
            highlighted_style
        } else {
            normal_style
        })
        .render(&mut f, chunks_right[0]);
    Block::default()
        .title("Diff")
        .borders(Borders::ALL)
        .border_style(if let UiWidget::Diff = model.current {
            highlighted_style
        } else {
            normal_style
        })
        .title_style(if let UiWidget::Diff = model.current {
            highlighted_style
        } else {
            normal_style
        })
        .render(&mut f, chunks_right[1]);

    let text = [
        Text::raw("abcdef\n"),
        Text::raw("abcdef\n"),
        Text::raw("abcdef\n"),
        Text::raw("abcdef\n"),
    ];

    Paragraph::new(text.iter())
        .block(
            Block::default()
                .title("Config")
                .borders(Borders::ALL)
                .border_style(if let UiWidget::Config = model.current {
                    highlighted_style
                } else {
                    normal_style
                })
                .title_style(if let UiWidget::Config = model.current {
                    highlighted_style
                } else {
                    normal_style
                }),
        )
        .wrap(true)
        .render(&mut f, chunks_right[2]);
}
