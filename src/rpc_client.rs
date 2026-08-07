use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct RpcForwarder {
    client: Client,
    upstream_url: String,
}

impl RpcForwarder {
    pub fn new(upstream_url: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to build async HTTP client"),
            upstream_url,
        }
    }

    pub async fn forward(&self, payload: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client
            .post(&self.upstream_url)
            .json(&payload)
            .send()
            .await?;

        let json_resp: Value = response.json().await?;
        Ok(json_resp)
    }
}
