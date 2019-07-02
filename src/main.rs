extern crate clap;

use clap::{App, Arg};
use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};

fn main() {
    let args = App::new("classrom-helper")
        .version("0.1.0")
        .author("Jiajie Chen <noc@jiegec.ac.cn>")
        .about("GitHub Classroom helper")
        .arg(
            Arg::with_name("organization")
                .short("o")
                .long("organization")
                .value_name("org")
                .help("GitHub organization name, e.g. physics-data")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("prefix")
                .short("p")
                .long("prefix")
                .value_name("prefix")
                .help("GitHub repo prefix, e.g. self-intro")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("students")
                .short("s")
                .long("students")
                .value_name("students")
                .help("Path to students csv, e.g. students.csv")
                .takes_value(true),
        )
        .get_matches();

    let org = args.value_of("org").unwrap_or("physics-data");
    let prefix = args.value_of("prefix").unwrap_or("self-intro");
    let students = args.value_of("students").unwrap_or("students.csv");

    let mut rdr = csv::Reader::from_reader(File::open(students).unwrap());
    for row in rdr.records() {
        let record = row.unwrap();
        let github = record.get(2).unwrap();
        let mut grade = false;
        if !Path::new("workspace")
            .join(format!("{}-{}", prefix, github))
            .join(".git")
            .exists()
        {
            println!("cloning {}", github);
            let output = Command::new("git")
                .current_dir("workspace")
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
            println!("pulling {}", github);
            let output = Command::new("git")
                .current_dir(format!("workspace/{}-{}", prefix, github))
                .arg("pull")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap();
            if output.success() {
                grade = true;
            } else {
                println!("pulling {:?} failed", output);
            }
        }
        if grade {
            println!("grading {}", github);
            let output = Command::new("python3")
                .current_dir(format!("workspace/{}-{}", prefix, github))
                .arg("grade.py")
                .stdout(Stdio::inherit())
                .stderr(Stdio::null())
                .status()
                .unwrap();
            println!("{:?}", output);
        }
    }

    println!("Hello, world!");
}
