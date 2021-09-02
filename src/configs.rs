use chrono::{DateTime, Utc};
use clap::{App, AppSettings, Arg};
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
        let args = App::new("classroom-helper")
            .about("GitHub Classroom helper")
            .author(crate_authors!())
            .version(crate_version!())
            .setting(AppSettings::ColoredHelp)
            .arg(
                Arg::with_name("organization")
                    .short("o")
                    .long("organization")
                    .value_name("org")
                    .help("GitHub organization name")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("prefix")
                    .short("p")
                    .long("prefix")
                    .value_name("prefix")
                    .help("GitHub repo prefix")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("students")
                    .short("s")
                    .long("students")
                    .value_name("students")
                    .help("Path to students csv")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("workspace")
                    .short("w")
                    .long("workspace")
                    .value_name("workspace")
                    .help("Path to workspace csv")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("template")
                    .short("t")
                    .long("template")
                    .value_name("template")
                    .help("Template repo slug")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("template_branch")
                    .short("b")
                    .long("template_branch")
                    .value_name("template_branch")
                    .help("Template repo branch")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("result")
                    .short("r")
                    .long("result")
                    .value_name("result")
                    .help("Result csv path")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("grader")
                    .short("g")
                    .long("grader")
                    .value_name("grader")
                    .help("Grader py name")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("before_grader")
                    .long("before_grader")
                    .value_name("before_grader")
                    .help("Anything to run before grader")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("config")
                    .value_name("config")
                    .help("Config file"),
            )
            .get_matches();

        let mut settings = config::Config::default();

        // Precedence
        // Commandline > Environment > Config

        if let Some(conf) = args.value_of("config") {
            settings.merge(config::File::with_name(conf)).unwrap();
        }
        settings
            .merge(config::Environment::with_prefix("CLASSROOM"))
            .unwrap();

        // Merge command line args
        let mut overwrite = false;
        let mut clap_args = config::Config::default();
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
        ]
        .iter()
        {
            if let Some(value) = args.value_of(key) {
                clap_args.set(*key, value).unwrap();
                overwrite = true;
            }
        }

        if overwrite {
            settings.merge(clap_args).unwrap();
        }

        let org = settings.get_str("organization").unwrap();
        let prefix = settings.get_str("prefix").unwrap();
        let students = settings.get_str("students").unwrap();
        let template = settings.get_str("template").unwrap();
        let template_branch = settings
            .get_str("template_branch")
            .unwrap_or(format!("master"));
        let workspace = settings.get_str("workspace").unwrap();
        let results = settings.get_str("result").unwrap();
        let grader = settings.get_str("grader").unwrap();
        let before_grader = settings.get_str("before_grader").ok();
        let diff = settings.get_str("diff").unwrap();
        let copy_values = settings.get_array("copy").unwrap();
        let deadline = settings
            .get_str("deadline")
            .ok()
            .and_then(|s| s.parse::<DateTime<Utc>>().ok());
        let mut copy = Vec::new();

        fs::create_dir_all(Path::new(&workspace)).unwrap();

        for value in copy_values.into_iter() {
            copy.push(value.into_str().unwrap());
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
