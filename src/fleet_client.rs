use hbb_common::config::Config;
use hbb_common::tokio;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
struct HeartbeatReq {
    id: String,
    hostname: String,
    os: String,
}

pub fn start() {
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;

            let url = Config::get_option("fleet-manager-url");
            let token = Config::get_option("fleet-manager-token");

            if url.is_empty() || token.is_empty() {
                continue;
            }

            let id = Config::get_id();
            let hostname = hbb_common::get_sys_name().unwrap_or_default();
            let os = std::env::consts::OS.to_string();

            let req = HeartbeatReq { id, hostname, os };

            let target_url = format!("{}/api/heartbeat", url.trim_end_matches('/'));
            
            let _ = client
                .post(&target_url)
                .bearer_auth(token)
                .json(&req)
                .send()
                .await;
        }
    });
}
