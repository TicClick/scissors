use std::{error::Error, process::Command};

pub enum OutputLines {
    First(usize),
    All,
}

pub fn output(
    program: &str,
    args: &[&str],
    line_count: OutputLines,
) -> Result<Vec<String>, Box<dyn Error>> {
    match Command::new(program).args(args).output() {
        Ok(output) => match String::from_utf8(output.stdout) {
            Ok(s) => {
                let lines = s.lines().map(|line| line.to_owned());
                match line_count {
                    OutputLines::First(n) => Ok(lines.take(n).collect()),
                    OutputLines::All => Ok(lines.collect()),
                }
            }
            Err(e) => Err(Box::new(e)),
        },
        Err(e) => Err(Box::new(e)),
    }
}

pub fn output_oneline(program: &str, args: &[&str]) -> Result<Option<String>, Box<dyn Error>> {
    output(program, args, OutputLines::First(1)).map(|lines| lines.first().cloned())
}

pub fn git(args: &[&str], line_count: OutputLines) -> Result<Vec<String>, Box<dyn Error>> {
    output("git", args, line_count)
}

pub fn git_oneline(args: &[&str]) -> Result<Option<String>, Box<dyn Error>> {
    output_oneline("git", args)
}
