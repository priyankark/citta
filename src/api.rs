use crate::{
    client::Client,
    config::Config,
    model::*,
    store::{Rejection, Store},
};
use anyhow::Result;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, Query, State, WebSocketUpgrade, ws::Message},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt, stream};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use subtle::ConstantTimeEq;
use tokio_util::sync::CancellationToken;

pub type Activity = Arc<Mutex<BTreeMap<String, usize>>>;
#[derive(Clone)]
pub struct App {
    pub config: Arc<Config>,
    pub store: Arc<Store>,
    pub token: Arc<String>,
    pub peers: Vec<Client>,
    pub enrollment: Option<Arc<crate::enrollment::Enrollment>>,
    pub inventory: crate::inventory::Inventory,
    pub managed_agents: crate::inventory::Managed,
    pub panel: crate::panel::Sessions,
    pub control_token: Arc<String>,
    pub control_enrollment: Option<Arc<crate::enrollment::Enrollment>>,
    pub permissions: Arc<crate::permissions::Permissions>,
    pub activity: Activity,
    pub shutdown: CancellationToken,
}
impl App {
    pub fn peer_clients(&self) -> Vec<Client> {
        let mut peers = self.peers.clone();
        if let Some(enrollment) = &self.enrollment {
            for client in enrollment.clients() {
                // An automatically refreshed credential takes precedence for the same identity.
                peers.retain(|p| p.node_id != client.node_id);
                peers.push(client);
            }
        }
        peers
    }
    pub fn origin_identity(&self, node: &str) -> Option<String> {
        if node == self.config.node_id {
            return Some(format!("local:{node}"));
        }
        self.enrollment
            .as_ref()
            .and_then(|e| e.principal(node))
            .or_else(|| {
                self.config
                    .peers
                    .iter()
                    .any(|p| p.node_id == node)
                    .then(|| format!("peer:{node}"))
            })
    }
    pub fn control_client(&self, node: &str) -> Option<Client> {
        if node == self.config.node_id {
            return Client::control_local(&self.config).ok();
        }
        self.control_enrollment
            .as_ref()?
            .clients()
            .into_iter()
            .find(|c| c.node_id == node)
    }
    pub fn execution_identity(&self, origin: &Client, task: &Task) -> Option<String> {
        if origin.node_id == self.config.node_id {
            Some(
                task.requester
                    .clone()
                    .unwrap_or_else(|| "legacy:unknown".into()),
            )
        } else {
            self.origin_identity(&origin.node_id)
        }
    }
    pub fn execution_allowed(&self, origin: &Client, task: &Task, capability: &str) -> bool {
        self.execution_identity(origin, task)
            .is_some_and(|id| self.permissions.allows(&id, capability))
    }
    pub async fn db<T: Send + 'static>(
        &self,
        f: impl FnOnce(&Store) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || f(&store)).await?
    }
    pub fn node_info(&self) -> NodeInfo {
        let active = self.activity.lock().unwrap_or_else(|e| e.into_inner());
        NodeInfo {
            permissions: Some(self.permissions.snapshot()),
            protocol: 1,
            node_id: self.config.node_id.clone(),
            version: env!("CARGO_PKG_VERSION").into(),
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            slots: self.config.slots,
            active: active.values().sum(),
            capabilities: self
                .config
                .capabilities
                .iter()
                .map(|c| CapabilityInfo {
                    harness: c.harness.clone(),
                    model: c.model.clone(),
                    name: c.name.clone(),
                    description: c.description.clone(),
                    slots: c.slots,
                    active: *active.get(&c.name).unwrap_or(&0),
                    max_timeout_secs: c.max_timeout_secs,
                    kind: c.kind,
                })
                .collect(),
            inventory: self
                .inventory
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone(),
            managed_agents: self
                .managed_agents
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .values()
                .cloned()
                .collect(),
        }
    }
}
pub fn router(app: App) -> Router {
    let enrollment = app.enrollment.clone();
    let panel = crate::panel::router(app.clone());
    let control = crate::control::router(app.clone());
    let control_enrollment = app.control_enrollment.clone();
    let router = Router::new()
        .route("/healthz", get(|| async { Json(json!({"ok":true})) }))
        .route("/v1/node", get(node))
        .route("/v1/discover", get(discover))
        .route("/v1/activity", get(activity))
        .route("/v1/enrollment", get(enrollment_status))
        .route("/v1/agents", get(discover))
        .route("/v1/agents/spawn", post(spawn_agent))
        .route("/v1/tasks", post(submit).get(list))
        .route("/v1/tasks/{id}", get(inspect))
        .route("/v1/tasks/{id}/cancel", post(cancel))
        .route("/v1/tasks/{id}/renew", post(renew))
        .route("/v1/tasks/{id}/complete", post(complete))
        .route("/v1/claim", post(claim))
        .route("/v1/events", get(events))
        .route("/v1/events/ws", get(events_ws))
        .layer(DefaultBodyLimit::max(MAX_BODY))
        .layer(middleware::from_fn_with_state(app.clone(), authenticate))
        .with_state(app)
        .merge(panel)
        .merge(control);
    let router = match control_enrollment {
        Some(e) => router.merge(crate::enrollment::router(e)),
        None => router,
    };
    match enrollment {
        Some(enrollment) => router.merge(crate::enrollment::router(enrollment)),
        None => router,
    }
}
#[derive(Clone)]
pub(crate) struct PeerIdentity(pub(crate) String);
async fn authenticate(
    State(app): State<App>,
    mut request: axum::extract::Request,
    next: Next,
) -> Response {
    // No browser use: reject Origin even with a valid token, preventing cross-origin WS use.
    if request.headers().contains_key(header::ORIGIN) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let supplied = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    if !bool::from(supplied.as_bytes().ct_eq(app.token.as_bytes())) {
        match app
            .enrollment
            .as_ref()
            .and_then(|e| e.authenticate(supplied))
        {
            Some(node_id) => {
                request.extensions_mut().insert(PeerIdentity(node_id));
            }
            None => return StatusCode::UNAUTHORIZED.into_response(),
        }
    }
    next.run(request).await
}
pub struct ApiError(anyhow::Error);
impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(e: E) -> Self {
        Self(e.into())
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self.0.downcast_ref::<Rejection>() {
            Some(Rejection::NotFound) => (StatusCode::NOT_FOUND, "task not found"),
            Some(Rejection::Conflict) => (
                StatusCode::CONFLICT,
                "state, lease, or idempotency conflict",
            ),
            Some(Rejection::Capacity) => (
                StatusCode::TOO_MANY_REQUESTS,
                "task capacity reached; drain or prune history",
            ),
            None => {
                tracing::error!(error = %self.0, "storage operation failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error")
            }
        };
        (status, Json(json!({"error":message}))).into_response()
    }
}
async fn node(State(app): State<App>) -> Json<NodeInfo> {
    Json(app.node_info())
}
async fn discover(State(app): State<App>) -> Json<Value> {
    let peers: Vec<Value> = stream::iter(app.peer_clients()).map(|peer| async move {
        match peer.get::<NodeInfo>("/v1/node").await {
            Ok(node) if node.node_id == peer.node_id && node.protocol == 1 => json!({"node_id":peer.node_id,"reachable":true,"node":node}),
            Ok(_) => json!({"node_id":peer.node_id,"reachable":false,"error":"node identity or protocol mismatch"}),
            Err(_) => json!({"node_id":peer.node_id,"reachable":false,"error":"peer unreachable or authentication failed"}),
        }
    }).buffer_unordered(8).collect().await;
    Json(
        json!({"local":app.node_info(),"peers":peers,"enrollment":app.enrollment.as_ref().map(|e|e.report())}),
    )
}
async fn enrollment_status(State(app): State<App>) -> Json<Value> {
    Json(
        app.enrollment
            .as_ref()
            .map_or_else(|| json!({"enabled":false}), |e| e.report()),
    )
}
async fn spawn_agent(
    State(app): State<App>,
    identity: Option<axum::Extension<PeerIdentity>>,
    Json(request): Json<SpawnAgent>,
) -> Result<Response, ApiError> {
    let submission = request.submission();
    if submission.validate().is_err() || request.prompt.trim().is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error":"invalid agent request"})),
        )
            .into_response());
    }
    let requester = identity
        .as_ref()
        .and_then(|i| app.origin_identity(&i.0.0))
        .unwrap_or_else(|| format!("local:{}", app.config.node_id));
    let retry = submission.clone();
    if let Some(task) = app
        .db(move |s| s.existing_submission(&retry, Some(&requester)))
        .await?
    {
        return Ok((StatusCode::CREATED, Json(task)).into_response());
    }
    let discovery = discover(State(app.clone())).await.0;
    let mut nodes = vec![discovery["local"].clone()];
    if let Some(peers) = discovery["peers"].as_array() {
        nodes.extend(
            peers
                .iter()
                .filter(|p| p["reachable"] == true)
                .map(|p| p["node"].clone()),
        );
    }
    let available = nodes.iter().any(|node| {
        request
            .target_node
            .as_ref()
            .is_none_or(|id| node["node_id"].as_str() == Some(id))
            && node["capabilities"].as_array().is_some_and(|caps| {
                caps.iter()
                    .any(|c| c["name"].as_str() == Some(&request.profile) && c["kind"] == "agent")
            })
    });
    if !available {
        return Ok((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error":"no reachable device advertises this agent profile"})),
        )
            .into_response());
    }
    submit(State(app), identity, Json(submission)).await
}

pub(crate) async fn overview(app: App) -> Result<Value> {
    let (discovery, activity) = tokio::join!(discover(State(app.clone())), activity(State(app)));
    Ok(
        json!({"generated_at":now(),"discovery":discovery.0,"activity":activity.map_err(|e| e.0)?.0}),
    )
}
async fn submit(
    State(app): State<App>,
    identity: Option<axum::Extension<PeerIdentity>>,
    Json(request): Json<Submission>,
) -> Result<Response, ApiError> {
    if let Err(e) = request.validate() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({"error":e.to_string()})),
        )
            .into_response());
    }
    let requester = match identity {
        Some(identity) => app
            .origin_identity(&identity.0.0)
            .ok_or_else(|| ApiError(anyhow::anyhow!("origin no longer trusted")))?,
        None => format!("local:{}", app.config.node_id),
    };
    let task = app
        .db(move |s| s.submit_as(request, now(), Some(requester)))
        .await?;
    Ok((StatusCode::CREATED, Json(task)).into_response())
}
#[derive(Deserialize)]
struct Listing {
    limit: Option<usize>,
}
async fn list(
    State(app): State<App>,
    Query(query): Query<Listing>,
) -> Result<Json<Value>, ApiError> {
    let tasks = app
        .db(move |s| s.list(query.limit.unwrap_or(50), i64::MAX))
        .await?;
    let summaries: Vec<Value> = tasks.into_iter().map(|t| json!({"id":t.id,"origin_node":t.origin_node,"capability":t.request.capability,"target_node":t.request.target_node,"status":t.status,"worker_node":t.worker_node,"created_at":t.created_at,"updated_at":t.updated_at})).collect();
    Ok(Json(json!(summaries)))
}
async fn inspect(State(app): State<App>, Path(id): Path<String>) -> Result<Json<Task>, ApiError> {
    Ok(Json(app.db(move |s| s.get(&id)).await?))
}
async fn cancel(State(app): State<App>, Path(id): Path<String>) -> Result<Json<Task>, ApiError> {
    Ok(Json(app.db(move |s| s.cancel(&id, now())).await?))
}
async fn claim(
    State(app): State<App>,
    identity: Option<axum::Extension<PeerIdentity>>,
    Json(request): Json<ClaimRequest>,
) -> Result<Response, ApiError> {
    if !valid_name(&request.worker_node) || !valid_name(&request.capability) {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    }
    if identity.is_some_and(|i| i.0.0 != request.worker_node) {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    if request.worker_node != app.config.node_id
        && !app
            .peer_clients()
            .iter()
            .any(|p| p.node_id == request.worker_node)
    {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    Ok(Json(app.db(move |s| s.claim(request, now())).await?).into_response())
}
async fn renew(
    State(app): State<App>,
    Path(id): Path<String>,
    Json(request): Json<LeaseRequest>,
) -> Result<Json<Task>, ApiError> {
    Ok(Json(
        app.db(move |s| s.renew(&id, &request.lease_token, now()))
            .await?,
    ))
}
async fn complete(
    State(app): State<App>,
    Path(id): Path<String>,
    Json(request): Json<Completion>,
) -> Result<Response, ApiError> {
    if !matches!(
        request.outcome.status,
        Status::Succeeded | Status::Failed | Status::Interrupted
    ) || request.outcome.stdout.len() > MAX_OUTPUT
        || request.outcome.stderr.len() > MAX_OUTPUT
        || request
            .outcome
            .error
            .as_ref()
            .is_some_and(|s| s.len() > 4096)
    {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    }
    Ok(Json(app.db(move |s| s.complete(&id, request, now())).await?).into_response())
}
#[derive(Deserialize)]
struct Cursor {
    #[serde(default)]
    after: i64,
}
async fn events(
    State(app): State<App>,
    Query(cursor): Query<Cursor>,
) -> Result<Json<Vec<Event>>, ApiError> {
    Ok(Json(app.db(move |s| s.events(cursor.after)).await?))
}
async fn events_ws(
    State(app): State<App>,
    Query(cursor): Query<Cursor>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.max_message_size(4096).max_frame_size(4096).on_upgrade(move |mut socket| async move {
        let mut after = cursor.after;
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        loop {
            tokio::select! {
                _ = app.shutdown.cancelled() => break,
                message = socket.recv() => match message { Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break, _ => {} },
                _ = interval.tick() => {
                    let Ok(events) = app.db(move |s| s.events(after)).await else { break };
                    for event in events {
                        after = event.sequence;
                        let Ok(text) = serde_json::to_string(&event) else { return };
                        if !matches!(tokio::time::timeout(Duration::from_secs(5), socket.send(Message::Text(text.into()))).await, Ok(Ok(()))) { return; }
                    }
                }
            }
        }
        let _ = socket.close().await;
    })
}

async fn activity(State(app): State<App>) -> Result<Json<Value>, ApiError> {
    let local = list(State(app.clone()), Query(Listing { limit: Some(20) }))
        .await?
        .0;
    let peers: Vec<Value> = stream::iter(app.peer_clients())
        .map(|peer| async move {
            let info = peer.get::<NodeInfo>("/v1/node").await;
            if !matches!(info, Ok(ref n) if n.node_id == peer.node_id && n.protocol == 1) {
                return json!({"node_id":peer.node_id,"reachable":false});
            }
            match peer.get::<Value>("/v1/tasks?limit=20").await {
                Ok(tasks) => json!({"node_id":peer.node_id,"reachable":true,"tasks":tasks}),
                Err(_) => json!({"node_id":peer.node_id,"reachable":false}),
            }
        })
        .buffer_unordered(16)
        .collect()
        .await;
    Ok(Json(
        json!({"local":{"node_id":app.config.node_id,"tasks":local},"peers":peers}),
    ))
}
