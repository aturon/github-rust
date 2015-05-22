/// Bindings for github pull request APIs

// Documentation References:
// https://developer.github.com/v3/pulls/

use Client;
use http;

/// Information provided from github about issues. Very incomplete!
#[derive(Debug, RustcDecodable)]
pub struct Pull {
    pub url: String,
    pub title: String,
}

pub fn list_pulls(client: &Client, user: &str, repo: &str) -> http::Result<Pull> {
    http::get(&client.user_agent,
              &format!("{}repos/{}/{}/pulls?per_page=100", client.base_url, user, repo),
              None)
}
