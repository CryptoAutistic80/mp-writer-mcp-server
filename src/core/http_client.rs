use std::time::Duration;

use reqwest::Client;

pub fn build_http_client(disable_proxy: bool) -> Result<Client, reqwest::Error> {
    let mut builder = Client::builder()
        .user_agent("deep-research-mcp-server/1.0")
        .timeout(Duration::from_secs(10));

    if disable_proxy {
        builder = builder.no_proxy();
    }

    builder.build()
}
