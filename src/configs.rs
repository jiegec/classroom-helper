use chrono::{DateTime, Utc};
use clap::{Arg, Command};
use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct Config {
    // organization
    pub org: String,
    // repo name prefix
    pub prefix: String,
    // path to students.csv
    pub students: String,
    // template repo name
    pub template: String,
    // template repo branch
    pub template_branch: String,
    // workspace dir name
    pub workspace: String,
    // result csv name
    pub results: String,
    // grader file name
    pub grader: String,
    // file to diff
    pub diff: String,
    // command to run before grader
    pub before_grader: Option<String>,
    // copy files from template
    pub copy: Vec<String>,
    // deadline
    pub deadline: Option<DateTime<Utc>>,
}

impl Config {
    pub fn new() -> Config {
        let args = Command::new("classroom-helper")
            .about("GitHub Classroom helper")
            .author(env!("CARGO_PKG_AUTHORS"))
            .version(env!("CARGO_PKG_VERSION"))
            .arg(
                Arg::new("organization")
                    .short('o')
                    .long("organization")
                    .value_name("org")
                    .help("GitHub organization name"),
            )
            .arg(
                Arg::new("prefix")
                    .short('p')
                    .long("prefix")
                    .value_name("prefix")
                    .help("GitHub repo prefix"),
            )
            .arg(
                Arg::new("students")
                    .short('s')
                    .long("students")
                    .value_name("students")
                    .help("Path to students csv"),
            )
            .arg(
                Arg::new("workspace")
                    .short('w')
                    .long("workspace")
                    .value_name("workspace")
                    .help("Path to workspace csv"),
            )
            .arg(
                Arg::new("template")
                    .short('t')
                    .long("template")
                    .value_name("template")
                    .help("Template repo slug"),
            )
            .arg(
                Arg::new("template_branch")
                    .short('b')
                    .long("template_branch")
                    .value_name("template_branch")
                    .help("Template repo branch"),
            )
            .arg(
                Arg::new("result")
                    .short('r')
                    .long("result")
                    .value_name("result")
                    .help("Result csv path"),
            )
            .arg(
                Arg::new("grader")
                    .short('g')
                    .long("grader")
                    .value_name("grader")
                    .help("Grader py name"),
            )
            .arg(
                Arg::new("before_grader")
                    .long("before_grader")
                    .value_name("before_grader")
                    .help("Anything to run before grader"),
            )
            .arg(Arg::new("config").value_name("config").help("Config file"))
            .get_matches();

        let mut builder = config::Config::builder();

        // Precedence
        // Commandline > Environment > Config

        if let Some(conf) = args.get_one::<String>("config") {
            builder = builder.add_source(config::File::with_name(conf));
        }
        builder = builder.add_source(config::Environment::with_prefix("CLASSROOM"));

        // Merge command line args
        for key in [
            "organization",
            "prefix",
            "students",
            "template",
            "template_branch",
            "workspace",
            "result",
            "grader",
            "before_grader",
        ] {
            if let Some(value) = args.get_one::<String>(key) {
                builder = builder.set_override(key, value.clone()).unwrap();
            }
        }

        let settings = builder.build().unwrap();

        let org = settings.get_string("organization").unwrap();
        let prefix = settings.get_string("prefix").unwrap();
        let students = settings.get_string("students").unwrap();
        let template = settings.get_string("template").unwrap();
        let template_branch = settings
            .get_string("template_branch")
            .unwrap_or_else(|_| "master".to_string());
        let workspace = settings.get_string("workspace").unwrap();
        let results = settings.get_string("result").unwrap();
        let grader = settings.get_string("grader").unwrap();
        let before_grader = settings.get_string("before_grader").ok();
        let diff = settings.get_string("diff").unwrap();
        let copy_values = settings.get_array("copy").unwrap();
        let deadline = settings
            .get_string("deadline")
            .ok()
            .and_then(|s| s.parse::<DateTime<Utc>>().ok());
        let mut copy = Vec::new();

        fs::create_dir_all(Path::new(&workspace)).unwrap();

        for value in copy_values.into_iter() {
            copy.push(value.into_string().unwrap());
        }

        Config {
            org,
            prefix,
            students,
            template,
            template_branch,
            workspace,
            results,
            grader,
            diff,
            copy,
            before_grader,
            deadline,
        }
    }
}
