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

#[derive(Debug, Copy, Clone, PartialEq)]
enum Team {
    Tools,
    Libs,
    Compiler,
    Lang,
}

impl Team {
    fn dir(&self) -> &str {
        match *self {
            Team::Tools => "tools",
            Team::Libs => "libs",
            Team::Compiler => "compiler",
            Team::Lang => "lang",
        }
    }

    fn label(&self) -> &str {
        match *self {
            Team::Tools => "T-tools",
            Team::Libs => "T-libs",
            Team::Compiler => "T-compiler",
            Team::Lang => "T-lang",
        }
    }
}

fn gen_rfcs(client: &Client, team: Team, file: &mut File) -> io::Result<()> {
    let (pulls, issues): (Vec<_>, Vec<_>) =
        list_issues(&client, "rust-lang", "rfcs", team.label())
            .unwrap().0.into_iter().rev()
            .partition(Issue::is_pr);

    try!(writeln!(file, "### RFC phase\n"));

    for issue in issues {
        if issue.labels.iter().any(|label| label.name == "P-high") {
            try!(writeln!(file, "- [Issue #{}]({}):", issue.number, issue.html_url));
            try!(writeln!(file, "  {}", issue.title));
        }
    }
    for pr in pulls {
        try!(writeln!(file, "- [PR #{}]({}):", pr.number, pr.html_url));
        try!(writeln!(file, "  {}", pr.title));
    }

    Ok(())
}

fn gen_issue_list(client: &Client, alt_repo: Option<&str>,
                  label: &str, file: &mut File, desc: &str) -> io::Result<()> {
    let issues = list_issues(&client, "rust-lang", alt_repo.unwrap_or("rust"), &label)
        .unwrap().0.into_iter().rev().filter(|i| !i.is_pr());

    for issue in issues {
        try!(writeln!(file, "- [{} #{}]({}):", desc, issue.number, issue.html_url));
        try!(writeln!(file, "  {}", issue.title));
    }
    Ok(())
}

fn gen_impl_phase(client: &Client, team: Team, file: &mut File) -> io::Result<()> {
    try!(writeln!(file, "### Implementation phase\n"));
    gen_issue_list(client, None, &format!("{},B-RFC-approved", team.label()), file, "Issue");
    gen_issue_list(client, None, &format!("{},final-comment-period", team.label()), file, "FCP PR ");
    Ok(())
}

fn gen_high_issues(client: &Client, team: Team, file: &mut File) -> io::Result<()> {
    try!(writeln!(file, "### High priority issues\n"));
    gen_issue_list(client, None, &format!("{},P-high", team.label()), file, "Issue");

    if team == Team::Tools {
        gen_issue_list(client, Some("Cargo"), "P-high", file, "Cargo Issue");
    }

    Ok(())
}

fn gen_needs_decision(client: &Client, team: Team, file: &mut File) -> io::Result<()> {
    try!(writeln!(file, "### Needs decision\n"));
    gen_issue_list(client, None, &format!("{},I-needs-decision", team.label()), file, "Issue");
    Ok(())
}

fn gen_reports() -> io::Result<()> {
    use Team::*;

    let args = parse_args();
    let path = PathBuf::from(args.flag_path.unwrap_or(".".to_string()));
    let date = args.flag_date.unwrap_or_else(|| {
        chrono::UTC::today().format("%Y-%m-%d").to_string()
    });
    let mut file_name = PathBuf::from(&date);
    file_name.set_extension("md");
    let client = &Client::new("aturon");

    for category in [Libs, Lang, Tools, Compiler].iter() {
        let full_path = path.join(category.dir()).join("reports").join(&file_name);
        if full_path.exists() && !args.flag_force {
            return Err(
                io::Error::new(io::ErrorKind::Other,
                               format!("The file `{}` already exists; use `-f` to overwrite.",
                                       full_path.display())));
        }

        let mut file = try!(File::create(full_path));
        try!(writeln!(&mut file, "# Subteam report: {} {}\n",
                      category.dir(), date));
        try!(writeln!(&mut file, "## Dashboard\n"));
        try!(gen_rfcs(&client, category.clone(), &mut file));
        try!(writeln!(&mut file, ""));
        try!(gen_impl_phase(&client, category.clone(), &mut file));
        try!(writeln!(&mut file, ""));
        try!(gen_high_issues(&client, category.clone(), &mut file));
        try!(writeln!(&mut file, ""));
        try!(gen_needs_decision(&client, category.clone(), &mut file));
    }

    Ok(())
}

fn main() {
    if let Err(e) = gen_reports() {
        println!("Error while generating reports:\n  {}", e)
    }
}
