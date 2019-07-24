use crate::configs::Config;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use termion::event::Key;
use threadpool::ThreadPool;

pub enum UiWidget {
    Student,
    Status,
    Log,
    Diff,
}

#[derive(Debug)]
pub enum Select {
    Whitebox,
    Blackbox,
    Last,
}

pub struct Student {
    pub student_id: String,
    pub name: String,
    pub github: String,
    pub blackbox: Option<f64>,
    pub whitebox: Option<f64>,
}

pub enum Message {
    Status(String),
    Grade((usize, Option<f64>)),
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

    pub grade_buffer: Option<String>,
    // last graded: grade, true for whitebox, false for blackbox
    pub last_grade: Option<(Option<f64>, bool)>,

    pub rx_messages: mpsc::Receiver<Message>,
    pub tx_messages: mpsc::Sender<Message>,

    pub grade_pool: ThreadPool,
    pub fetch_pool: ThreadPool,
}

impl Model {
    fn git_fetch(&self, repo: String, branch: String) {
        let tx = self.tx_messages.clone();
        let config = self.config.clone();
        self.fetch_pool.execute(move || {
            let mut reset = false;
            if !Path::new(&config.workspace)
                .join(&repo)
                .join(".git")
                .exists()
            {
                tx.send(Message::Status(format!("Cloning {} begin", repo)))
                    .unwrap();
                let output = Command::new("git")
                    .current_dir(&config.workspace)
                    .arg("clone")
                    .arg(format!("git@github.com:{}/{}.git", config.org, repo))
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .unwrap();
                if output.success() {
                    reset = true;
                    tx.send(Message::Status(format!("Cloning {} done", repo)))
                        .unwrap();
                } else {
                    tx.send(Message::Status(format!("Cloning {} failed", repo)))
                        .unwrap();
                }
            } else {
                tx.send(Message::Status(format!("Fetching {} begin", repo)))
                    .unwrap();
                let output = Command::new("git")
                    .current_dir(format!("{}/{}", config.workspace, repo))
                    .arg("fetch")
                    .arg("origin")
                    .arg(&branch)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .unwrap();
                if output.success() {
                    reset = true;
                    tx.send(Message::Status(format!("Fetching {} done", repo)))
                        .unwrap();
                } else {
                    tx.send(Message::Status(format!("Fetching {} failed", repo)))
                        .unwrap();
                }
            }
            if reset {
                let output = Command::new("git")
                    .current_dir(format!("{}/{}", config.workspace, repo))
                    .arg("clean")
                    .arg("-f")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .unwrap();
                if output.success() {
                    let output = Command::new("git")
                        .current_dir(format!("{}/{}", config.workspace, repo))
                        .arg("reset")
                        .arg(format!("origin/{}", branch))
                        .arg("--hard")
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .unwrap();
                    if output.success() {
                        let output = Command::new("git")
                            .current_dir(format!("{}/{}", config.workspace, repo))
                            .arg("checkout")
                            .arg(branch)
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status()
                            .unwrap();
                        if !output.success() {
                            tx.send(Message::Status(format!("Checkout {} failed", repo)))
                                .unwrap();
                        }
                    } else {
                        tx.send(Message::Status(format!("Resetting {} failed", repo)))
                            .unwrap();
                    }
                } else {
                    tx.send(Message::Status(format!("Resetting {} failed", repo)))
                        .unwrap();
                }
            }
        });
    }

    fn git_grade(&self, index: usize, github: String) {
        let tx = self.tx_messages.clone();
        let config = self.config.clone();
        self.grade_pool.execute(move || {
            if Path::new(&config.workspace)
                .join(format!("{}-{}", config.prefix, github))
                .join(".git")
                .exists()
            {
                for path in config.copy.iter() {
                    let orig_path = Path::new(&config.workspace)
                        .join(&config.template)
                        .join(&path);
                    if orig_path.is_dir() {
                        let dest_path = Path::new(&config.workspace)
                            .join(format!("{}-{}", config.prefix, github))
                            .join(&path);
                        fs_extra::dir::remove(&dest_path).unwrap();
                        let mut options = fs_extra::dir::CopyOptions::new();
                        options.overwrite = true;
                        options.copy_inside = true;
                        fs_extra::dir::copy(orig_path, dest_path, &options).unwrap();
                    } else if orig_path.is_file() {
                        let mut options = fs_extra::file::CopyOptions::new();
                        options.overwrite = true;
                        fs_extra::file::copy(
                            orig_path,
                            Path::new(&config.workspace)
                                .join(format!("{}-{}", config.prefix, github))
                                .join(&path),
                            &options,
                        )
                        .unwrap();
                    }
                }

                let interpreter = if config.grader.ends_with("sh") {
                    "bash"
                } else {
                    "python3"
                };

                tx.send(Message::Status(format!("Grading {} begin", github)))
                    .unwrap();
                let output = Command::new(interpreter)
                    .current_dir(format!(
                        "{}/{}-{}",
                        &config.workspace, &config.prefix, github
                    ))
                    .arg(&config.grader)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap();
                let res = output.wait_with_output().unwrap();
                let ans = String::from_utf8_lossy(&res.stdout);
                let grade = if let Ok(value) = serde_json::from_str::<Value>(&ans.trim()) {
                    if let Some(g) = value.get("grade") {
                        g.as_f64()
                    } else {
                        None
                    }
                } else {
                    None
                };
                tx.send(Message::Status(format!(
                    "Grading {} ended with {:?}",
                    github, grade
                )))
                .unwrap();
                tx.send(Message::Grade((index, grade))).unwrap();
            } else {
                tx.send(Message::Status(format!(
                    "Grading {} repo not found",
                    github
                )))
                .unwrap();
                tx.send(Message::Grade((index, None))).unwrap();
            }
        });
    }

    // true for whitebox, false for blackbox
    fn update_grade(&mut self, select: Select) {
        if let Some(index) = self.student_select {
            let (new_grade, selector) = if let Select::Last = select {
                if let Some(last) = self.last_grade {
                    last
                } else {
                    return;
                }
            } else {
                let new_grade = if let Some(grade) = &self.grade_buffer {
                    grade.parse::<f64>().ok()
                } else {
                    None
                };
                let selector = if let Select::Whitebox = select {
                    true
                } else {
                    false
                };
                (new_grade, selector)
            };

            if selector {
                self.students[index].whitebox = new_grade;
            } else {
                self.students[index].blackbox = new_grade;
            }
            if index + 1 < self.students.len() {
                self.student_select = Some(index + 1);
            }
            self.last_grade = Some((new_grade, selector));
        }
    }
    fn gen_results(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        // UTF-8 BOM
        buffer.push(0xef);
        buffer.push(0xbb);
        buffer.push(0xbf);
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
        buffer
    }
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

        let (tx, rx) = mpsc::channel();

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

            grade_buffer: None,
            last_grade: None,

            tx_messages: tx,
            rx_messages: rx,

            grade_pool: ThreadPool::new(1),
            fetch_pool: ThreadPool::new(4),
        }
    }

    pub fn handle(&mut self, key: Key) {
        let orig_student_select = self.student_select;
        let mut update_grade = false;
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
                    "       g G: grade blackbox for selected(g)/all(G) students\n"
                ));
                self.status
                    .push(format!("       s d: save(s)/diff(d) results\n"));
                self.status.push(format!(
                    "       [num]+b w: set blackbox(b)/whitebox(b) grade manually\n"
                ));
                self.status
                    .push(format!("       r: repeat last grade for current student\n"));
                self.status
                    .push(format!("       t: bump template repo to newest version\n"));
            }
            Key::Char('d') => {
                let results = if Path::new(&self.config.results).is_file() {
                    &self.config.results
                } else {
                    "/dev/null"
                };
                let mut spawn = Command::new("git")
                    .arg("diff")
                    .arg("--no-index")
                    .arg("--minimal")
                    .arg(results)
                    .arg("-")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap();

                let buffer = self.gen_results();

                spawn.stdin.as_mut().unwrap().write(&buffer).unwrap();
                let out = spawn.wait_with_output().unwrap();
                self.diff = String::from_utf8(out.stdout).unwrap().replace("\t", "    ");
                if self.diff.len() > 0 {
                    self.diff_lines = self.diff.chars().filter(|ch| *ch == '\n').count();
                    self.diff_scroll_start = 0;
                } else {
                    self.diff = format!("No difference");
                    self.diff_lines = 1;
                    self.diff_scroll_start = 0;
                }
            }
            Key::Char('s') => {
                let buffer = self.gen_results();

                let mut file = File::create(&self.config.results).unwrap();
                file.write_all(&buffer).unwrap();
                self.status
                    .push(format!("Saved to {}\n", self.config.results));
            }
            Key::Char(ch) if (ch >= '0' && ch <= '9') || ch == '.' => {
                if let Some(buffer) = &mut self.grade_buffer {
                    buffer.push(ch);
                } else {
                    self.grade_buffer = Some(ch.to_string());
                }
                update_grade = true;
            }
            Key::Char('b') => {
                self.update_grade(Select::Blackbox);
            }
            Key::Char('w') => {
                self.update_grade(Select::Whitebox);
            }
            Key::Char('r') => {
                self.update_grade(Select::Last);
            }
            Key::Char('f') => {
                if let Some(index) = self.student_select {
                    self.git_fetch(
                        format!(
                            "{}-{}",
                            self.config.prefix,
                            self.students[index].github.clone(),
                        ),
                        format!("master"),
                    );
                }
            }
            Key::Char('F') => {
                self.git_fetch(
                    self.config.template.clone(),
                    self.config.template_branch.clone(),
                );
                for stu in self.students.iter() {
                    self.git_fetch(
                        format!("{}-{}", self.config.prefix, stu.github.clone()),
                        format!("master"),
                    );
                }
            }
            Key::Char('g') => {
                if let Some(index) = self.student_select {
                    self.git_grade(index, self.students[index].github.clone());
                }
            }
            Key::Char('G') => {
                for (index, stu) in self.students.iter().enumerate() {
                    self.git_grade(index, stu.github.clone());
                }
            }
            Key::Char('t') => {
                self.git_fetch(
                    self.config.template.clone(),
                    self.config.template_branch.clone(),
                );
            }
            _ => {
                self.status.push(format!("Unhandled key {:?}\n", key));
            }
        }

        if !update_grade {
            self.grade_buffer = None;
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

                // user
                let output = Command::new("git")
                    .current_dir(format!(
                        "{}/{}-{}",
                        self.config.workspace, self.config.prefix, student.github
                    ))
                    .arg("log")
                    .arg("--pretty=format:%H")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .output()
                    .unwrap();
                let commits = String::from_utf8(output.stdout).unwrap();
                let user_commits: Vec<&str> = commits.split("\n").collect();

                // template
                let output = Command::new("git")
                    .current_dir(format!(
                        "{}/{}",
                        self.config.workspace, self.config.template
                    ))
                    .arg("log")
                    .arg("--pretty=format:%H")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .output()
                    .unwrap();
                let commits = String::from_utf8(output.stdout).unwrap();
                let template_commits: Vec<&str> = commits.split("\n").collect();

                // clear first
                self.diff = String::from("No common ancestor commit found");
                self.diff_lines = 1;
                self.diff_scroll_start = 0;

                // find common ancestor
                for commit in user_commits {
                    if let Some(ancestor) = template_commits.iter().find(|c| *c == &commit) {
                        let output = Command::new("git")
                            .current_dir(format!(
                                "{}/{}-{}",
                                self.config.workspace, self.config.prefix, student.github
                            ))
                            .arg("diff")
                            .arg(ancestor)
                            .arg("HEAD")
                            .stdout(Stdio::piped())
                            .stderr(Stdio::null())
                            .output()
                            .unwrap();
                        self.diff = String::from_utf8_lossy(&output.stdout)
                            .to_owned()
                            .replace("\t", "    ");
                        self.diff_lines = self.diff.chars().filter(|ch| *ch == '\n').count();
                        self.diff_scroll_start = 0;
                        break;
                    }
                }
            } else {
                self.log = String::from("N/A");
                self.log_lines = 1;
                self.log_scroll_start = 0;

                self.diff = String::from("N/A");
                self.diff_lines = 1;
                self.diff_scroll_start = 0;
            }
        }
    }

    pub fn tick(&mut self) {
        while let Ok(message) = self.rx_messages.try_recv() {
            match message {
                Message::Status(status) => {
                    self.status.push(format!(
                        "{} ({} jobs left)\n",
                        status,
                        self.fetch_pool.queued_count() + self.grade_pool.queued_count()
                    ));
                }
                Message::Grade((index, grade)) => {
                    self.students[index].blackbox = grade;
                }
            }
        }
    }
}
