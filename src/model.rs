use crate::configs::Config;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use termion::event::Key;

pub enum UiWidget {
    Student,
    Status,
    Log,
    Diff,
}

pub struct Student {
    pub student_id: String,
    pub name: String,
    pub github: String,
    pub blackbox: Option<f64>,
    pub whitebox: Option<f64>,
}

pub struct Model {
    pub config: Config,
    pub current: UiWidget,
    pub students: Vec<Student>,
    pub status: Vec<String>,

    pub log: String,
    pub diff: String,

    pub student_select: Option<usize>,
    pub student_render_start: usize,

    pub log_scroll_start: usize,
    pub log_lines: usize,
    pub diff_scroll_start: usize,
    pub diff_lines: usize,
}

impl Model {
    pub fn new(config: Config) -> Model {
        let mut status = Vec::new();

        // read students
        let mut rdr = csv::Reader::from_reader(File::open(&config.students).unwrap());
        let mut students = Vec::new();
        for row in rdr.records() {
            // cols: student_id, name, github
            let record = row.unwrap();
            let student_id = record.get(0).unwrap();
            let name = record.get(1).unwrap();
            let github = record.get(2).unwrap();
            students.push(Student {
                student_id: String::from(student_id),
                name: String::from(name),
                github: String::from(github),
                blackbox: None,
                whitebox: None,
            });
        }

        // read existed results
        if Path::new(&config.results).exists() {
            let mut rdr = csv::Reader::from_reader(File::open(&config.results).unwrap());
            for row in rdr.records() {
                // cols: student_id, name, github, blackbox, whitebox
                let record = row.unwrap();
                let student_id = record.get(0).unwrap();
                let name = record.get(1).unwrap();
                let github = record.get(2).unwrap();
                for stu in students.iter_mut() {
                    if stu.student_id == student_id && stu.name == name && stu.github == github {
                        if let Some(blackbox) = record.get(3) {
                            if let Ok(grade) = blackbox.parse() {
                                stu.blackbox = Some(grade);
                            } else {
                                stu.blackbox = None;
                            }
                        }
                        if let Some(whitebox) = record.get(4) {
                            if let Ok(grade) = whitebox.parse() {
                                stu.whitebox = Some(grade);
                            } else {
                                stu.whitebox = None;
                            }
                        }
                        break;
                    }
                }
            }
        }

        status.push(format!("Read {} students from data\n", students.len()));

        Model {
            config,
            current: UiWidget::Student,
            students,
            status,
            student_select: None,
            student_render_start: 0,

            log: String::new(),
            diff: String::new(),

            log_scroll_start: 0,
            log_lines: 0,
            diff_scroll_start: 0,
            diff_lines: 0,
        }
    }

    pub fn handle(&mut self, key: Key) {
        let orig_student_select = self.student_select;
        match key {
            // change current widget
            Key::Char('H') => {
                self.current = match self.current {
                    UiWidget::Student => UiWidget::Student,
                    UiWidget::Status => UiWidget::Status,
                    UiWidget::Log => UiWidget::Student,
                    UiWidget::Diff => UiWidget::Student,
                };
            }
            Key::Char('J') => {
                self.current = match self.current {
                    UiWidget::Student => UiWidget::Status,
                    UiWidget::Status => UiWidget::Status,
                    UiWidget::Log => UiWidget::Diff,
                    UiWidget::Diff => UiWidget::Diff,
                };
            }
            Key::Char('K') => {
                self.current = match self.current {
                    UiWidget::Student => UiWidget::Student,
                    UiWidget::Status => UiWidget::Student,
                    UiWidget::Log => UiWidget::Log,
                    UiWidget::Diff => UiWidget::Log,
                };
            }
            Key::Char('L') => {
                self.current = match self.current {
                    UiWidget::Student => UiWidget::Log,
                    UiWidget::Status => UiWidget::Diff,
                    UiWidget::Log => UiWidget::Log,
                    UiWidget::Diff => UiWidget::Diff,
                };
            }
            Key::Char('j') => {
                match self.current {
                    UiWidget::Student => {
                        self.student_select = match self.student_select {
                            None => {
                                if self.students.len() > 0 {
                                    Some(0)
                                } else {
                                    None
                                }
                            }
                            Some(current) => {
                                if self.students.len() > current + 1 {
                                    Some(current + 1)
                                } else {
                                    Some(0)
                                }
                            }
                        }
                    }
                    UiWidget::Log => {
                        self.log_scroll_start = if self.log_scroll_start + 1 < self.log_lines {
                            self.log_scroll_start + 1
                        } else {
                            0
                        };
                    }
                    UiWidget::Diff => {
                        self.diff_scroll_start = if self.diff_scroll_start + 1 < self.diff_lines {
                            self.diff_scroll_start + 1
                        } else {
                            0
                        };
                    }
                    _ => {}
                };
            }
            Key::Char('k') => {
                match self.current {
                    UiWidget::Student => {
                        self.student_select = match self.student_select {
                            None => {
                                if self.students.len() > 0 {
                                    Some(self.students.len() - 1)
                                } else {
                                    None
                                }
                            }
                            Some(current) => {
                                if current > 0 {
                                    Some(current - 1)
                                } else {
                                    Some(self.students.len() - 1)
                                }
                            }
                        }
                    }
                    UiWidget::Log => {
                        self.log_scroll_start = if self.log_scroll_start > 0 {
                            self.log_scroll_start - 1
                        } else {
                            self.log_lines - 1
                        };
                    }
                    UiWidget::Diff => {
                        self.diff_scroll_start = if self.diff_scroll_start > 0 {
                            self.diff_scroll_start - 1
                        } else {
                            self.diff_lines - 1
                        };
                    }
                    _ => {}
                };
            }
            Key::Char('h') | Key::Char('?') => {
                self.status.push(format!("Usage: \n"));
                self.status
                    .push(format!("       H J K L: navigate between panels\n"));
                self.status.push(format!("       j k: scroll in panels\n"));
                self.status
                    .push(format!("       f F: fetch selected(f)/all(F) students\n"));
                self.status.push(format!(
                    "       b B: grade blackbox for selected(b)/all(B) students\n"
                ));
                self.status
                    .push(format!("       s d: save(s)/diff(d) results\n"));
            }
            Key::Char('d') => {
                let mut spawn = Command::new("git")
                    .arg("diff")
                    .arg("--no-index")
                    .arg("--minimal")
                    .arg(&self.config.results)
                    .arg("-")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap();

                let mut buffer = Vec::new();
                let mut wtr = csv::Writer::from_writer(&mut buffer);
                wtr.write_record(&["学号", "姓名", "GitHub", "黑盒成绩", "白盒成绩"])
                    .unwrap();
                for stu in self.students.iter() {
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

                    wtr.write_record(&[
                        &stu.student_id,
                        &stu.name,
                        &stu.github,
                        &blackbox,
                        &whitebox,
                    ])
                    .unwrap();
                }
                wtr.flush().unwrap();
                drop(wtr);

                spawn.stdin.as_mut().unwrap().write(&buffer).unwrap();
                let out = spawn.wait_with_output().unwrap();
                self.diff = String::from_utf8(out.stdout).unwrap().replace("\t", "    ");
                self.diff_lines = self.diff.chars().filter(|ch| *ch == '\n').count();
                self.diff_scroll_start = 0;
            }
            _ => {
                self.status.push(format!("Unhandled key {:?}\n", key));
            }
        }

        if orig_student_select != self.student_select {
            // Selection changed
            let student = &self.students[self.student_select.unwrap()];
            self.status.push(format!("Looking at {}\n", student.name));

            if Path::new(&self.config.workspace)
                .join(format!("{}-{}", self.config.prefix, student.github))
                .join(".git")
                .exists()
            {
                let output = Command::new("git")
                    .current_dir(format!(
                        "{}/{}-{}",
                        self.config.workspace, self.config.prefix, student.github
                    ))
                    .arg("log")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .output()
                    .unwrap();
                self.log = String::from_utf8(output.stdout)
                    .unwrap()
                    .replace("\t", "    ");
                self.log_lines = self.log.chars().filter(|ch| *ch == '\n').count();
                self.log_scroll_start = 0;

                let output = Command::new("git")
                    .current_dir(&self.config.workspace)
                    .arg("diff")
                    .arg("--no-index")
                    .arg("--minimal")
                    .arg(format!("{}/{}", self.config.template, self.config.diff))
                    .arg(format!(
                        "{}-{}/{}",
                        self.config.prefix, student.github, self.config.diff
                    ))
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .output()
                    .unwrap();
                self.diff = String::from_utf8(output.stdout)
                    .unwrap()
                    .replace("\t", "    ");
                self.diff_lines = self.diff.chars().filter(|ch| *ch == '\n').count();
                self.diff_scroll_start = 0;
            }
        }
    }
}
