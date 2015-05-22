#![feature(path_ext)]

extern crate github;
extern crate docopt;
extern crate rustc_serialize;
extern crate chrono;

use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::fs::File;

use github::Client;
use github::issues::{Issue, list_issues};
use github::pulls::list_pulls;

use docopt::Docopt;

static USAGE: &'static str = "
Usage: subteam-reports [options]

Options:
    --path PATH         Location of subteam repository (defaults to cwd)
    --date DATE         Date to use for reports (defaults to today)
    -f, --force         Overwrite any existing reports
";

#[derive(RustcDecodable)]
struct Args {
    flag_path: Option<String>,
    flag_date: Option<String>,
    flag_force: bool,
}

fn parse_args() -> Args {
    Docopt::new(USAGE)
        .and_then(|d| d.help(true).decode())
        .unwrap_or_else(|e| e.exit())
}

fn gen_rfcs(client: &Client, category: &str, file: &mut File) -> io::Result<()> {
    let label = format!("T-{}", category);
    let (pulls, issues): (Vec<_>, Vec<_>) =
        list_issues(&client, "rust-lang", "rfcs", &label)
            .unwrap().0.into_iter().rev()
            .partition(Issue::is_pr);

    try!(writeln!(file, "### RFC phase\n"));

    for issue in issues {
        if issue.labels.iter().any(|label| label.name == "P-high") {
            try!(writeln!(file, "- [Issue]({}):", issue.url));
            try!(writeln!(file, "  {}", issue.title));
        }
    }
    for pr in pulls {
        try!(writeln!(file, "- [PR]({}):", pr.url));
        try!(writeln!(file, "  {}", pr.title));
    }

    Ok(())
}

fn gen_issues(client: &Client, category: &str, file: &mut File) -> io::Result<()> {
    let label = format!("T-{},B-RFC-approved", category);
    let issues = list_issues(&client, "rust-lang", "rust", &label)
        .unwrap().0.into_iter().rev().filter(|i| !i.is_pr());

    try!(writeln!(file, "### Implementation phase\n"));

    for issue in issues {
        try!(writeln!(file, "- [Issue]({}):", issue.url));
        try!(writeln!(file, "  {}", issue.title));
    }

    Ok(())
}

fn gen_reports() -> io::Result<()> {
    let args = parse_args();
    let path = PathBuf::from(args.flag_path.unwrap_or(".".to_string()));
    let date = args.flag_date.unwrap_or_else(|| {
        chrono::UTC::today().format("%Y-%m-%d").to_string()
    });
    let mut file_name = PathBuf::from(&date);
    file_name.set_extension("md");
    let client = &Client::new("aturon");

    for category in ["libs", "lang", "compiler", "tools"].iter() {
        let full_path = path.join(&category).join("reports").join(&file_name);
        if full_path.exists() && !args.flag_force {
            return Err(
                io::Error::new(io::ErrorKind::Other,
                               format!("The file `{}` already exists; use `-f` to overwrite.",
                                       full_path.display())));
        }

        let mut file = try!(File::create(full_path));
        try!(writeln!(&mut file, "# Subteam report: {} {}\n",
                      category, date));
        try!(writeln!(&mut file, "## Dashboard\n"));
        try!(gen_rfcs(&client, category, &mut file));
        try!(writeln!(&mut file, ""));
        try!(gen_issues(&client, category, &mut file));
    }

    Ok(())
}

fn main() {
    if let Err(e) = gen_reports() {
        println!("Error while generating reports:\n  {}", e)
    }
}
