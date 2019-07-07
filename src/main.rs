#[macro_use]
extern crate clap;
extern crate config;

use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use std::io;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Paragraph, Text, Widget};
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

    /*
    if !Path::new(&workspace).join(&template).join(".git").exists() {
        println!("cloning {}", template);
        let output = Command::new("git")
            .current_dir(&workspace)
            .arg("clone")
            .arg(format!("git@github.com:{}/{}.git", org, template))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        if output.success() {
            println!("cloning {:?} done", template);
        } else {
            println!("cloning {:?} failed", template);
        }
    }

    println!("pulling {}", template);
    let output = Command::new("git")
        .current_dir(format!("{}/{}", &workspace, template))
        .arg("pull")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    if output.success() {
        println!("cloning {:?} done", template);
    } else {
        println!("pulling {:?} failed", template);
    }

    let mut rdr = csv::Reader::from_reader(File::open(students).unwrap());
    let mut results = File::create(results).unwrap();
    results.write(&[0xef, 0xbb, 0xbf]).unwrap();
    let mut wtr = csv::Writer::from_writer(results);
    wtr.write_record(&["学号", "姓名", "GitHub", "成绩"])
        .unwrap();
    for row in rdr.records() {
        let record = row.unwrap();
        let github = record.get(2).unwrap();
        let mut grade = false;
        if !Path::new(&workspace)
            .join(format!("{}-{}", prefix, github))
            .join(".git")
            .exists()
        {
            println!("cloning {}", github);
            let output = Command::new("git")
                .current_dir(&workspace)
                .arg("clone")
                .arg(format!("git@github.com:{}/{}-{}.git", org, prefix, github))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap();
            if output.success() {
                grade = true;
            } else {
                println!("cloning {:?} failed", output);
            }
        } else {
            if args.occurrences_of("fetch") > 0 {
                println!("fetching {}", github);
                let output = Command::new("git")
                    .current_dir(format!("{}/{}-{}", workspace, prefix, github))
                    .arg("fetch")
                    .arg("origin")
                    .arg("master")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .unwrap();
                if output.success() {
                    grade = true;
                } else {
                    println!("fetching {:?} failed", output);
                }
            } else {
                grade = true;
            }
        }

        if grade {
            let output = Command::new("git")
                .current_dir(format!("{}/{}-{}", &workspace, prefix, github))
                .arg("reset")
                .arg("origin/master")
                .arg("--hard")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap();
            if !output.success() {
                println!("reseting {:?} failed", github);
            }

            let mut options = fs_extra::file::CopyOptions::new();
            options.overwrite = true;
            fs_extra::file::copy(
                Path::new(&workspace).join(&template).join(&grader),
                Path::new(&workspace)
                    .join(format!("{}-{}", prefix, github))
                    .join(&grader),
                &options,
            )
            .unwrap();

            println!("grading {}", github);
            let output = Command::new("python3")
                .current_dir(format!("{}/{}-{}", workspace, prefix, github))
                .arg(&grader)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let res = output.wait_with_output().unwrap();
            let ans = String::from_utf8_lossy(&res.stdout);
            let value: Value = serde_json::from_str(&ans).unwrap();
            let grade = value.get("grade").unwrap().as_f64().unwrap();
            println!("grade {:?}", grade);
            wtr.write_record(&[
                record.get(0).unwrap(),
                record.get(1).unwrap(),
                record.get(2).unwrap(),
                &format!("{}", grade),
            ])
            .unwrap();
        } else {
            println!("unable to grade {}", github);
            wtr.write_record(&[
                record.get(0).unwrap(),
                record.get(1).unwrap(),
                record.get(2).unwrap(),
                "N/A",
            ])
            .unwrap();
        }
        wtr.flush().unwrap();
    }
    wtr.flush().unwrap();

    */
    Ok(())
}
