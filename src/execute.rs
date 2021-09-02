use std::path::Path;
use std::process::{Command, Stdio};

pub fn run<'a, PP: AsRef<Path>>(script: &'a str, pwd: PP) -> String {
    let interpreter = if script.ends_with("sh") {
        "bash"
    } else {
        "python3"
    };
    let output = Command::new(interpreter)
        .current_dir(pwd)
        .arg(script)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let res = output.wait_with_output().unwrap();
    String::from_utf8_lossy(&res.stdout).to_string()
}
