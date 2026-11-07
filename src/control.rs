//! Owner management API. Task tokens cannot authenticate these routes.
use crate::{
    api::App,
    permissions::{Snapshot, Update},
    store::Rejection,
};
use axum::{
    Extension, Json, Router,
    extract::{DefaultBodyLimit, Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::{Value, json};
use subtle::ConstantTimeEq;
#[derive(Clone)]
struct Identity {
    actor: String,
    local: bool,
}
pub fn router(app: App) -> Router {
    Router::new()
        .route("/v1/control/permissions", get(read).post(update))
        .route("/v1/panel/session", post(session))
        .layer(DefaultBodyLimit::max(32 * 1024))
        .layer(middleware::from_fn_with_state(app.clone(), authenticate))
        .with_state(app)
}
async fn authenticate(State(app): State<App>, mut request: Request, next: Next) -> Response {
    if request.headers().contains_key(header::ORIGIN) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    let identity = if bool::from(token.as_bytes().ct_eq(app.control_token.as_bytes())) {
        Some(Identity {
            actor: format!("local:{}", app.config.node_id),
            local: true,
        })
    } else {
        app.control_enrollment
            .as_ref()
            .and_then(|e| e.authenticate(token).and_then(|id| e.principal(&id)))
            .map(|actor| Identity {
                actor,
                local: false,
            })
    };
    let Some(identity) = identity else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    request.extensions_mut().insert(identity);
    next.run(request).await
}
async fn session(State(app): State<App>, Extension(identity): Extension<Identity>) -> Response {
    if !identity.local {
        return StatusCode::FORBIDDEN.into_response();
    }
    match app.panel.issue() {
        Some(code) => Json(
            json!({"url":format!("{}/#login={code}",app.config.local_url()),"network_url":app.enrollment.as_ref().and_then(|e|e.public_url()).map(|url|format!("{url}/#login={code}")),"expires_in_secs":120}),
        )
        .into_response(),
        None => StatusCode::TOO_MANY_REQUESTS.into_response(),
    }
}
pub async fn view(app: &App) -> anyhow::Result<Value> {
    let snapshot = app.permissions.snapshot();
    let mut origins = vec![
        json!({"id":format!("local:{}",app.config.node_id),"node_id":app.config.node_id,"local":true}),
    ];
    for peer in app.peer_clients() {
        if let Some(id) = app.origin_identity(&peer.node_id) {
            origins.push(json!({"id":id,"node_id":peer.node_id,"local":false}));
        }
    }
    let history = app.db(|s| s.permission_history()).await?;
    Ok(
        json!({"node_id":app.config.node_id,"snapshot":snapshot,"origins":origins,"capabilities":app.node_info().capabilities,"history":history}),
    )
}
async fn read(State(app): State<App>) -> Response {
    match view(&app).await {
        Ok(v) => Json(v).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
pub async fn apply(
    app: &App,
    request: Update,
    actor: String,
) -> Result<Snapshot, (StatusCode, &'static str)> {
    if request.policy.validate().is_err() {
        return Err((StatusCode::BAD_REQUEST, "Invalid device permissions"));
    }
    let permissions = app.permissions.clone();
    tokio::task::spawn_blocking(move || permissions.update(request, actor))
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Permissions could not be saved",
            )
        })?
        .map_err(|e| {
            if e.downcast_ref::<Rejection>().is_some() {
                (
                    StatusCode::CONFLICT,
                    "Permissions changed elsewhere. Reload before saving.",
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Permissions could not be saved",
                )
            }
        })
}
async fn update(
    State(app): State<App>,
    Extension(identity): Extension<Identity>,
    Json(request): Json<Update>,
) -> Response {
    match apply(&app, request, identity.actor).await {
        Ok(v) => Json(v).into_response(),
        Err((status, message)) => (status, Json(json!({"error":message}))).into_response(),
    }
}
