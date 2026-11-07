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
    #[cfg(test)]
    pub(crate) fn test_endpoint(&mut self, ca: &[u8], address: std::net::IpAddr) -> Result<()> {
        let url = reqwest::Url::parse(&self.url)?;
        self.http = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(4))
            .tls_certs_only([reqwest::Certificate::from_der(ca)?])
            .resolve(
                url.host_str().unwrap(),
                std::net::SocketAddr::new(address, url.port_or_known_default().unwrap()),
            )
            .build()?;
        Ok(())
    }
    pub fn local(config: &Config) -> Result<Self> {
        Self::new(
            &config.local_url(),
            read_token(&config.token_file)?,
            config.node_id.clone(),
        )
    }
    pub fn control_local(config: &Config) -> Result<Self> {
        Self::new(
            &config.local_url(),
            read_token(&config.data_dir.join("control.token"))?,
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
        if matches!(
            path,
            "/v1/discover" | "/v1/activity" | "/v1/agents" | "/v1/agents/spawn"
        ) {
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
        if !status.is_success() {
            return Err(RemoteStatus(status.as_u16()).into());
        }
        serde_json::from_slice(&bytes).context("invalid peer response")
    }
}

#[derive(Debug)]
pub struct RemoteStatus(pub u16);
impl std::fmt::Display for RemoteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "peer returned HTTP {}", self.0)
    }
}
impl std::error::Error for RemoteStatus {}
