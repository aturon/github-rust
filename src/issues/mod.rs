use Client;

use error::*;
use response::Response;

use rustc_serialize::Decoder;
use rustc_serialize::Decodable;

use http;

use std::fmt;

/// Documentation References:
/// https://developer.github.com/v3/activity/

#[derive(Debug, RustcDecodable)]
pub struct Issue {
    pub url: String,
    pub title: String,
    pub html_url: String,
}

pub fn list_issues(client: &Client, user: &str, repo: &str, labels: &str)
                          -> http::Result<Issue> {
    http::get(&client.user_agent,
              &format!("{}repos/{}/{}/issues?per_page=100&labels={}",
                       client.base_url, user, repo, labels),
              None)
}
