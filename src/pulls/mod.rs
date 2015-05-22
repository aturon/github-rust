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
pub struct Pull {
    pub url: String,
    pub title: String,
}

pub fn list_pull_requests(client: &Client, user: &str, repo: &str) -> http::Result<Pull> {
    http::get(&client.user_agent,
              &format!("{}repos/{}/{}/pulls?per_page=100", client.base_url, user, repo),
              None)
}
