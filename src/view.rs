use crate::model::{Model, UiWidget};
use std::io;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::terminal::Frame;
use tui::widgets::{Block, Borders, Paragraph, Row, Table, Text, Widget};
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

    // Students
    let mut students = Vec::new();
    let highlighted_row_style = Style::default().bg(Color::Gray);
    for (index, stu) in model
        .students
        .iter()
        .enumerate()
        .skip(model.student_render_start)
    {
        let blackbox = if let Some(grade) = stu.blackbox {
            grade.to_string()
        } else {
            format!("N/A")
        };
        let whitebox = if let Some(grade) = stu.whitebox {
            grade.to_string()
        } else {
            format!("N/A")
        };
        if Some(index) == model.student_select {
            students.push(Row::StyledData(
                vec![
                    stu.student_id.clone(),
                    stu.name.clone(),
                    stu.github.clone(),
                    blackbox,
                    whitebox,
                ]
                .into_iter(),
                highlighted_row_style,
            ))
        } else {
            students.push(Row::Data(
                vec![
                    stu.student_id.clone(),
                    stu.name.clone(),
                    stu.github.clone(),
                    blackbox,
                    whitebox,
                ]
                .into_iter(),
            ))
        }
    }

    Table::new(
        [
            "Student Id",
            "Name",
            "GitHub",
            "Blackbox Grade",
            "Whitebox Grade",
        ]
        .into_iter(),
        students.into_iter(),
    )
    .block(
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
            }),
    )
    .widths(&[15, 10, 10, 15, 15])
    .render(&mut f, chunks_left[0]);

    // Status
    let mut status = Vec::new();
    for line in model.status.iter() {
        status.push(Text::raw(line.clone()));
    }
    Paragraph::new(status.iter())
        .block(
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
                }),
        )
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
