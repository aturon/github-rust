extern crate github;

use github::Client;
use github::error::*;
use github::issues::*;

fn list_rfc_prs(client: &Client, label: &str) {
    let issues = list_issues(&client, "rust-lang", "rfcs", label).unwrap().0;

    println!("{}:\n", label);
    for issue in issues {
        if issue.html_url.contains("/pull/") {
            println!("- [PR]({}):", issue.url);
            println!("  {}", issue.title);
        }
    }
    println!("");
}

fn main() {
    let client = &Client::new("aturon");

    for category in ["T-libs", "T-lang", "T-compiler", "T-tools"].iter() {
        list_rfc_prs(&client, category)
    }
}
