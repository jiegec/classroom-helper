use crate::model::{Model, UiWidget};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::terminal::Frame;
use tui::widgets::{Block, Borders, Paragraph, Row, Table, Text, Widget};

pub fn draw<B: Backend>(model: &mut Model, mut f: &mut Frame<B>) {
    let highlighted_style = Style::default().fg(Color::Gray);
    let normal_style = Style::default();

    let chunks_virt = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let chunks_left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
        .split(chunks_virt[0]);

    // Students
    if let Some(select) = model.student_select {
        if select < model.student_render_start {
            model.student_render_start = select;
        } else if select > model.student_render_start + (chunks_left[0].height as usize - 6) {
            model.student_render_start = select - (chunks_left[0].height as usize - 6);
        }
    }
    let mut students = Vec::new();
    let highlighted_row_style = Style::default().bg(Color::Gray);
    let mut github_width = 10;
    for (index, stu) in model
        .students
        .iter()
        .enumerate()
        .skip(model.student_render_start)
    {
        github_width = std::cmp::max(github_width, stu.github.len());

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
    .widths(&[15, 10, github_width as u16, 15, 15])
    .render(&mut f, chunks_left[0]);

    // Status
    let mut status = Vec::new();
    for line in model.status.iter() {
        status.push(Text::raw(line.clone()));
    }
    let status_scroll = if status.len() > chunks_left[1].height as usize - 3 {
        status.len() - (chunks_left[1].height as usize - 3)
    } else {
        0
    };
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
        .scroll(status_scroll as u16)
        .render(&mut f, chunks_left[1]);

    let chunks_right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks_virt[1]);

    // Log
    Paragraph::new([Text::raw(model.log.clone())].iter())
        .block(
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
                }),
        )
        .scroll(model.log_scroll_start as u16)
        .wrap(true)
        .render(&mut f, chunks_right[0]);

    // Diff
    Paragraph::new([Text::raw(model.diff.clone())].iter())
        .block(
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
                }),
        )
        .scroll(model.diff_scroll_start as u16)
        .wrap(true)
        .render(&mut f, chunks_right[1]);
}
