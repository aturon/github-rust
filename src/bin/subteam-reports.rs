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

use docopt::Docopt;

static USAGE: &'static str = "
Usage: subteam-reports [options] <team>...

Options:
    --path PATH         Location of subteam repository (defaults to cwd)
    --date DATE         Date to use for reports (defaults to today)
    -f, --force         Overwrite any existing reports
    -h, --help          Print this message
";

#[derive(RustcDecodable)]
struct Args {
    arg_team: Vec<String>,
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

const DIRS: &'static [(Team, &'static str)] = &[
    (Team::Tools, "tools"),
    (Team::Libs, "libs"),
    (Team::Compiler, "compiler"),
    (Team::Lang, "lang"),
    ];

impl Team {
    fn with_name(s: &str) -> Result<Team, &str> {
        match
            DIRS.iter()
                .filter(|&&(_, dir)| dir == s)
                .map(|&(team, _)| team)
                .next()
        {
            Some(v) => Ok(v),
            None => Err(s),
        }
    }

    fn dir(self) -> &'static str {
        DIRS.iter()
            .filter(|&&(team, _)| team == self)
            .map(|&(_, dir)| dir)
            .next()
            .unwrap()
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
    let (fcps, normal_pulls): (Vec<_>, Vec<_>) = pulls.into_iter()
        .partition(|i| i.labels.iter().any(|lbl| lbl.name == "final-comment-period"));

    try!(writeln!(file, "### RFC phase\n"));

    for pr in fcps {
        try!(writeln!(file, "- [FCP PR #{}]({}):", pr.number, pr.html_url));
        try!(writeln!(file, "  {}", pr.title));
    }
    for pr in normal_pulls {
        try!(writeln!(file, "- [PR #{}]({}):", pr.number, pr.html_url));
        try!(writeln!(file, "  {}", pr.title));
    }
    for issue in issues {
        if issue.labels.iter().any(|label| label.name == "P-high") {
            try!(writeln!(file, "- [Issue #{}]({}):", issue.number, issue.html_url));
            try!(writeln!(file, "  {}", issue.title));
        }
    }

    Ok(())
}

fn gen_issue_list(client: &Client, alt_repo: Option<&str>,
                  label: &str, file: &mut File, desc: &str, issues_only: bool) -> io::Result<()> {
    let all_issues = list_issues(&client, "rust-lang", alt_repo.unwrap_or("rust"), &label)
        .unwrap().0.into_iter().rev().filter(|i| !issues_only || !i.is_pr());
    let (fcps, normal_issues): (Vec<_>, Vec<_>) = all_issues.into_iter()
        .partition(|i| i.labels.iter().any(|lbl| lbl.name == "final-comment-period"));

    for issue in fcps {
        try!(writeln!(file, "- [FCP {} #{}]({}):", desc, issue.number, issue.html_url));
        try!(writeln!(file, "  {}", issue.title));
    }
    for issue in normal_issues {
        try!(writeln!(file, "- [{} #{}]({}):", desc, issue.number, issue.html_url));
        try!(writeln!(file, "  {}", issue.title));
    }
    Ok(())
}

fn gen_impl_phase(client: &Client, team: Team, file: &mut File) -> io::Result<()> {
    try!(writeln!(file, "### Implementation phase\n"));
    try!(gen_issue_list(client, None, &format!("{},B-RFC-approved", team.label()),
                        file, "Issue", true));
    Ok(())
}

fn gen_stab_phase(client: &Client, team: Team, file: &mut File) -> io::Result<()> {
    try!(writeln!(file, "### Stabilization phase\n"));
    try!(gen_issue_list(client, None, &format!("{},B-unstable", team.label()),
                        file, "Issue", true));
    Ok(())
}

fn gen_high_issues(client: &Client, team: Team, file: &mut File) -> io::Result<()> {
    try!(writeln!(file, "### High priority issues\n"));
    try!(gen_issue_list(client, None, &format!("{},P-high", team.label()),
                        file, "Issue", true));

    if team == Team::Tools {
        try!(gen_issue_list(client, Some("Cargo"), "P-high",
                            file, "Cargo Issue", true));
    }

    Ok(())
}

fn gen_needs_decision(client: &Client, team: Team, file: &mut File) -> io::Result<()> {
    try!(writeln!(file, "### Issues needing a decision\n"));
    try!(gen_issue_list(client, None, &format!("{},I-needs-decision", team.label()),
                        file, "Issue", false));
    Ok(())
}

type SectionGen = fn(&Client, Team, &mut File) -> io::Result<()>;

fn gen_sections(client: &Client, team: Team, file: &mut File, sections: &[SectionGen]) -> io::Result<()> {
    for section in sections {
        try!(section(client, team, file));
        try!(writeln!(file, ""));
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

    let categories: Result<Vec<Team>, &str> =
        args.arg_team
            .iter()
            .map(|s| Team::with_name(s))
            .collect();
    let categories = match categories {
        Ok(v) => v,
        Err(bad_name) => {
            return Err(
                io::Error::new(io::ErrorKind::Other,
                               format!("unrecognized team name `{}`", bad_name)));
        }
    };

    for category in categories {
        let full_path = path.join(category.dir()).join("reports").join(&file_name);
        if full_path.exists() && !args.flag_force {
            println!("Warning: The file `{}` already exists; use `-f` to overwrite.",
                     full_path.display());
            continue;
        }

        let mut file = try!(File::create(full_path));
        try!(writeln!(&mut file, "# Subteam report: {} {}\n",
                      category.dir(), date));
        try!(writeln!(&mut file, "## Highlights\n"));

        try!(writeln!(&mut file, "## Dashboard\n"));

        try!(gen_sections(&client, category, &mut file, &[
            gen_high_issues,
            gen_needs_decision,
            gen_rfcs,
            gen_impl_phase,
            gen_stab_phase,
        ]));
    }

    Ok(())
}

fn main() {
    if let Err(e) = gen_reports() {
        println!("Error while generating reports:\n  {}", e)
    }
}
