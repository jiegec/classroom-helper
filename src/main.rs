#[macro_use]
extern crate clap;

use clap::{App, Arg};
use serde_json::Value;
use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};
use std::io::Write;

fn main() {
    let args = App::new("classrom-helper")
        .about("GitHub Classroom helper")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("organization")
                .short("o")
                .long("organization")
                .value_name("org")
                .help("GitHub organization name")
                .default_value("physics-data")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("prefix")
                .short("p")
                .long("prefix")
                .value_name("prefix")
                .help("GitHub repo prefix")
                .default_value("self-intro")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("students")
                .short("s")
                .long("students")
                .value_name("students")
                .help("Path to students csv")
                .default_value("students.csv")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("workspace")
                .short("w")
                .long("workspace")
                .value_name("workspace")
                .help("Path to workspace csv")
                .default_value("workspace")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("template")
                .short("t")
                .long("template")
                .value_name("template")
                .help("Template repo slug")
                .default_value("tpl_self-introduction")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("fetch")
                .short("f")
                .long("fetch")
                .help("Fetch new commits or not"),
        )
        .get_matches();

    let org = args.value_of("organization").unwrap();
    let prefix = args.value_of("prefix").unwrap();
    let students = args.value_of("students").unwrap();
    let template = args.value_of("template").unwrap();
    let workspace = args.value_of("workspace").unwrap();

    if !Path::new(workspace).join(template).join(".git").exists() {
        println!("cloning {}", template);
        let output = Command::new("git")
            .current_dir(workspace)
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
        .current_dir(format!("{}/{}", workspace, template))
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
    let mut results = File::create("results.csv").unwrap();
    results.write(&[0xef, 0xbb, 0xbf]).unwrap();
    let mut wtr = csv::Writer::from_writer(results);
    wtr.write_record(&["学号","姓名","GitHub","成绩"]).unwrap();
    for row in rdr.records() {
        let record = row.unwrap();
        let github = record.get(2).unwrap();
        let mut grade = false;
        if !Path::new(workspace)
            .join(format!("{}-{}", prefix, github))
            .join(".git")
            .exists()
        {
            println!("cloning {}", github);
            let output = Command::new("git")
                .current_dir(workspace)
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
                .current_dir(format!("{}/{}-{}", workspace, prefix, github))
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
                Path::new(workspace).join(template).join("grade.py"),
                Path::new(workspace)
                    .join(format!("{}-{}", prefix, github))
                    .join("grade.py"),
                &options,
            )
            .unwrap();


            println!("grading {}", github);
            let output = Command::new("python3")
                .current_dir(format!("{}/{}-{}", workspace, prefix, github))
                .arg("grade.py")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let res = output.wait_with_output().unwrap();
            let ans = String::from_utf8_lossy(&res.stdout);
            let value: Value = serde_json::from_str(&ans).unwrap();
            let grade = value.get("grade").unwrap().as_f64().unwrap();
            println!("grade {:?}", grade);
            wtr.write_record(&[record.get(0).unwrap(), record.get(1).unwrap(), record.get(2).unwrap(), &format!("{}", grade)]).unwrap();
        } else {
            println!("unable to grade {}", github);
            wtr.write_record(&[record.get(0).unwrap(), record.get(1).unwrap(), record.get(2).unwrap(), "N/A"]).unwrap();
        }
        wtr.flush().unwrap();
    }
    wtr.flush().unwrap();
}
