/// Bindings for github issue APIs

// Documentation References:
// https://developer.github.com/v3/issues/

use Client;
use http;

/// Information provided from github about issues. Very incomplete!
#[derive(Debug, RustcDecodable)]
pub struct Issue {
    pub number: u64,
    pub url: String,
    pub title: String,
    pub html_url: String,
    pub labels: Vec<Label>,
}

/// Information provided from github about issue labels. Very incomplete!
#[derive(Debug, RustcDecodable)]
pub struct Label {
    pub name: String,
}

pub fn list_issues(client: &Client, user: &str, repo: &str, labels: &str)
                          -> http::Result<Issue> {
    http::get(&client.user_agent,
              &format!("{}repos/{}/{}/issues?per_page=100&labels={}",
                       client.base_url, user, repo, labels),
              None)
}

impl Issue {
    pub fn is_pr(&self) -> bool {
        self.html_url.contains("/pull/")
    }
}
