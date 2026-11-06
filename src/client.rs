use crate::{
    config::{Config, read_token, validate_url},
    model::MAX_BODY,
};
use anyhow::{Context, Result, ensure};
use futures_util::StreamExt;
use serde::{Serialize, de::DeserializeOwned};
use std::{sync::Arc, time::Duration};

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    url: String,
    token: Arc<String>,
    pub node_id: String,
}
impl Client {
    pub fn new(url: &str, token: String, node_id: String) -> Result<Self> {
        validate_url(url)?;
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(4))
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .build()?;
        Ok(Self {
            http,
            url: url.trim_end_matches('/').into(),
            token: Arc::new(token),
            node_id,
        })
    }
    pub fn local(config: &Config) -> Result<Self> {
        Self::new(
            &config.local_url(),
            read_token(&config.token_file)?,
            config.node_id.clone(),
        )
    }
    pub fn peers(config: &Config) -> Result<Vec<Self>> {
        config
            .peers
            .iter()
            .map(|p| Self::new(&p.url, read_token(&p.token_file)?, p.node_id.clone()))
            .collect()
    }
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request(reqwest::Method::GET, path, None::<&()>).await
    }
    pub async fn post<T: DeserializeOwned>(&self, path: &str, body: &impl Serialize) -> Result<T> {
        self.request(reqwest::Method::POST, path, Some(body)).await
    }
    async fn request<T: DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T> {
        let mut req = self
            .http
            .request(method, format!("{}{path}", self.url))
            .bearer_auth(self.token.as_str());
        if path == "/v1/discover" || path == "/v1/activity" {
            req = req.timeout(Duration::from_secs(40));
        }
        if let Some(body) = body {
            req = req.json(body);
        }
        let response = req.send().await.context("peer request failed")?;
        let status = response.status();
        let mut stream = response.bytes_stream();
        let mut bytes = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            ensure!(
                bytes.len() + chunk.len() <= MAX_BODY,
                "peer response exceeds limit"
            );
            bytes.extend_from_slice(&chunk);
        }
        ensure!(
            status.is_success(),
            "peer returned HTTP {}",
            status.as_u16()
        );
        serde_json::from_slice(&bytes).context("invalid peer response")
    }
}
