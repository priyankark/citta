//! Local owner control panel. Browser sessions never contain mesh tokens.
use crate::{
    api::{self, App},
    model::now,
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct Store {
    codes: BTreeMap<String, i64>,
    sessions: BTreeMap<String, i64>,
}
#[derive(Clone, Default)]
pub struct Sessions(Arc<Mutex<Store>>);
fn secret() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
impl Sessions {
    pub fn issue(&self) -> Option<String> {
        let mut store = self.0.lock().unwrap_or_else(|e| e.into_inner());
        store.codes.retain(|_, expires| *expires > now());
        if store.codes.len() >= 16 {
            return None;
        }
        let code = secret();
        store.codes.insert(digest(&code), now() + 120);
        Some(code)
    }
    fn login(&self, code: &str) -> Option<String> {
        if code.len() != 64 {
            return None;
        }
        let mut store = self.0.lock().unwrap_or_else(|e| e.into_inner());
        store.sessions.retain(|_, expires| *expires > now());
        let expires = store.codes.remove(&digest(code))?;
        if expires <= now() || store.sessions.len() >= 16 {
            return None;
        }
        let session = secret();
        store.sessions.insert(digest(&session), now() + 8 * 3600);
        Some(session)
    }
    fn authenticated(&self, headers: &HeaderMap) -> bool {
        let Some(cookie) = headers.get(header::COOKIE).and_then(|s| s.to_str().ok()) else {
            return false;
        };
        let token = cookie
            .split(';')
            .filter_map(|v| v.trim().split_once('='))
            .find(|(key, _)| *key == "citta_panel")
            .map(|(_, v)| v);
        token.is_some_and(|token| {
            token.len() == 64
                && self
                    .0
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .sessions
                    .get(&digest(token))
                    .is_some_and(|expires| *expires > now())
        })
    }
}
fn local_request(app: &App, headers: &HeaderMap) -> bool {
    let host = headers.get(header::HOST).and_then(|v| v.to_str().ok());
    let local = app.config.local_url();
    let network = app.enrollment.as_ref().and_then(|e| e.public_url());
    let network_host = network
        .as_deref()
        .and_then(|url| url.strip_prefix("https://"));
    let known_host = host == Some(app.config.listen.to_string().as_str())
        || (host.is_some() && host == network_host);
    known_host
        && headers.get(header::ORIGIN).is_none_or(|v| {
            v.to_str()
                .is_ok_and(|origin| origin == local || network.as_deref() == Some(origin))
        })
        && headers
            .get("sec-fetch-site")
            .is_none_or(|v| matches!(v.to_str(), Ok("same-origin" | "none")))
}
fn secure_cookie(headers: &HeaderMap) -> &'static str {
    if headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("https://"))
    {
        "; Secure"
    } else {
        ""
    }
}
fn secured(response: impl IntoResponse) -> Response {
    let mut response = response.into_response();
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    headers.insert("content-security-policy","default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'".parse().unwrap());
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert("referrer-policy", "no-referrer".parse().unwrap());
    response
}
pub fn router(app: App) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/panel/app.js", get(js))
        .route("/panel/style.css", get(css))
        .route("/panel/login", post(login))
        .route("/panel/overview", get(overview))
        .route("/panel/task", get(task))
        .route("/panel/logout", post(logout))
        .route(
            "/panel/permissions",
            get(permissions).post(save_permissions),
        )
        .layer(DefaultBodyLimit::max(32 * 1024))
        .with_state(app)
}
async fn index() -> Response {
    secured((
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("../ui/index.html"),
    ))
}
async fn js() -> Response {
    secured((
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("../ui/app.js"),
    ))
}
async fn css() -> Response {
    secured((
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../ui/style.css"),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Login {
    code: String,
}
async fn login(State(app): State<App>, headers: HeaderMap, Json(request): Json<Login>) -> Response {
    if !local_request(&app, &headers) {
        return secured(StatusCode::FORBIDDEN);
    }
    let Some(session) = app.panel.login(&request.code) else {
        return secured(StatusCode::UNAUTHORIZED);
    };
    let mut response = secured(Json(json!({"ok":true})));
    response.headers_mut().insert(
        header::SET_COOKIE,
        format!(
            "citta_panel={session}; HttpOnly; SameSite=Strict; Path=/panel; Max-Age=28800{}",
            secure_cookie(&headers)
        )
        .parse()
        .unwrap(),
    );
    response
}
async fn overview(State(app): State<App>, headers: HeaderMap) -> Response {
    if !local_request(&app, &headers) {
        return secured(StatusCode::FORBIDDEN);
    }
    if !app.panel.authenticated(&headers) {
        return secured(StatusCode::UNAUTHORIZED);
    }
    match api::overview(app).await {
        Ok(value) => secured(Json(value)),
        Err(_) => secured((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error":"Network status temporarily unavailable"})),
        )),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskQuery {
    node: String,
    id: uuid::Uuid,
}
async fn task(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<TaskQuery>,
) -> Response {
    if !local_request(&app, &headers) {
        return secured(StatusCode::FORBIDDEN);
    }
    if !app.panel.authenticated(&headers) {
        return secured(StatusCode::UNAUTHORIZED);
    }
    let client = if query.node == app.config.node_id {
        crate::client::Client::local(&app.config).ok()
    } else {
        app.peer_clients()
            .into_iter()
            .find(|p| p.node_id == query.node)
    };
    let Some(client) = client else {
        return secured(StatusCode::NOT_FOUND);
    };
    let info = client.get::<crate::model::NodeInfo>("/v1/node").await;
    if !matches!(info,Ok(ref node) if node.node_id==query.node && node.protocol==1) {
        return secured(StatusCode::BAD_GATEWAY);
    }
    match client
        .get::<crate::model::Task>(&format!("/v1/tasks/{}", query.id))
        .await
    {
        Ok(task) => secured(Json(task)),
        Err(_) => secured(StatusCode::BAD_GATEWAY),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeviceQuery {
    node: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PermissionEdit {
    node: String,
    expected_revision: i64,
    policy: crate::permissions::Policy,
}
fn proxy_error(error: anyhow::Error) -> Response {
    let status = error
        .downcast_ref::<crate::client::RemoteStatus>()
        .and_then(|s| StatusCode::from_u16(s.0).ok())
        .unwrap_or(StatusCode::BAD_GATEWAY);
    let message = match status {
        StatusCode::CONFLICT => "Permissions changed elsewhere. Reload before saving.",
        StatusCode::NOT_FOUND => "Upgrade this device to configure permissions.",
        _ => {
            "Device management is unavailable. Your changes were not confirmed; reload before retrying."
        }
    };
    secured((status, Json(json!({"error":message}))))
}
async fn permissions(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<DeviceQuery>,
) -> Response {
    if !local_request(&app, &headers) {
        return secured(StatusCode::FORBIDDEN);
    }
    if !app.panel.authenticated(&headers) {
        return secured(StatusCode::UNAUTHORIZED);
    }
    if query.node == app.config.node_id {
        return match crate::control::view(&app).await {
            Ok(v) => secured(Json(v)),
            Err(e) => proxy_error(e),
        };
    }
    let Some(client) = app.control_client(&query.node) else {
        return secured((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(
                json!({"error":"This device has not established a management connection. Both devices need the permissions-enabled build."}),
            ),
        ));
    };
    match client
        .get::<serde_json::Value>("/v1/control/permissions")
        .await
    {
        Ok(v) if v["node_id"].as_str() == Some(&query.node) => secured(Json(v)),
        Ok(_) => secured(StatusCode::BAD_GATEWAY),
        Err(e) => proxy_error(e),
    }
}
async fn save_permissions(
    State(app): State<App>,
    headers: HeaderMap,
    Json(edit): Json<PermissionEdit>,
) -> Response {
    if !local_request(&app, &headers)
        || headers.get("x-citta-panel").and_then(|v| v.to_str().ok()) != Some("1")
    {
        return secured(StatusCode::FORBIDDEN);
    }
    if !app.panel.authenticated(&headers) {
        return secured(StatusCode::UNAUTHORIZED);
    }
    let request = crate::permissions::Update {
        expected_revision: edit.expected_revision,
        policy: edit.policy,
    };
    if request.policy.validate().is_err() {
        return secured(StatusCode::BAD_REQUEST);
    }
    if edit.node == app.config.node_id {
        return match crate::control::apply(&app, request, format!("local:{}", app.config.node_id))
            .await
        {
            Ok(v) => secured(Json(v)),
            Err((status, message)) => secured((status, Json(json!({"error":message})))),
        };
    }
    let Some(client) = app.control_client(&edit.node) else {
        return secured((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error":"Device management is unavailable. No changes were sent."})),
        ));
    };
    // Authenticate the destination identity before any write; only registered peer URLs are used.
    match client
        .get::<serde_json::Value>("/v1/control/permissions")
        .await
    {
        Ok(v) if v["node_id"].as_str() == Some(&edit.node) => {}
        Ok(_) => return secured(StatusCode::BAD_GATEWAY),
        Err(e) => return proxy_error(e),
    }
    match client
        .post::<crate::permissions::Snapshot>("/v1/control/permissions", &request)
        .await
    {
        Ok(v) => secured(Json(v)),
        Err(e) => proxy_error(e),
    }
}
async fn logout(State(app): State<App>, headers: HeaderMap) -> Response {
    if !local_request(&app, &headers) {
        return secured(StatusCode::FORBIDDEN);
    }
    if let Some(cookie) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok())
        && let Some((_, token)) = cookie
            .split(';')
            .filter_map(|v| v.trim().split_once('='))
            .find(|(key, _)| *key == "citta_panel")
    {
        app.panel
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .sessions
            .remove(&digest(token));
    }
    let mut response = secured(Json(json!({"ok":true})));
    response.headers_mut().insert(
        header::SET_COOKIE,
        format!(
            "citta_panel=; HttpOnly; SameSite=Strict; Path=/panel; Max-Age=0{}",
            secure_cookie(&headers)
        )
        .parse()
        .unwrap(),
    );
    response
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn login_codes_are_single_use_and_only_hashed_sessions_are_retained() {
        let sessions = Sessions::default();
        let code = sessions.issue().unwrap();
        let session = sessions.login(&code).unwrap();
        assert!(sessions.login(&code).is_none());
        assert!(!sessions.0.lock().unwrap().sessions.contains_key(&session));
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            format!("citta_panel={session}").parse().unwrap(),
        );
        assert!(sessions.authenticated(&headers));
        sessions
            .0
            .lock()
            .unwrap()
            .sessions
            .insert(digest(&session), now() - 1);
        assert!(!sessions.authenticated(&headers));
    }
    #[test]
    fn expired_and_unknown_login_codes_fail_closed() {
        let sessions = Sessions::default();
        let code = sessions.issue().unwrap();
        sessions
            .0
            .lock()
            .unwrap()
            .codes
            .insert(digest(&code), now() - 1);
        assert!(sessions.login(&code).is_none());
        assert!(sessions.login(&secret()).is_none());
    }
}
