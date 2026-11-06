use citta::{
    api::{App, router},
    config::Config,
    model::{Event, Submission},
    store::Store,
};
use futures_util::StreamExt;
use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn authenticated_events_replay_from_durable_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path(), "test", 6, 100, 100).unwrap());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let token = "a".repeat(64);
    let stop = CancellationToken::new();
    let app = App {
        config: Arc::new(Config {
            node_id: "test".into(),
            listen: address,
            data_dir: dir.path().into(),
            token_file: dir.path().join("token"),
            slots: 1,
            lease_secs: 6,
            max_pending: 100,
            max_history: 100,
            peers: vec![],
            capabilities: vec![],
        }),
        store: store.clone(),
        token: Arc::new(token.clone()),
        peers: vec![],
        activity: Arc::new(Mutex::new(BTreeMap::new())),
        shutdown: stop.clone(),
    };
    let stopped = stop.clone();
    let server = tokio::spawn(async move {
        axum::serve(listener, router(app))
            .with_graceful_shutdown(stopped.cancelled_owned())
            .await
            .unwrap()
    });
    let task = store
        .submit(
            Submission {
                capability: "echo".into(),
                input: json!({}),
                idempotency_key: "event-test".into(),
                target_node: None,
                timeout_secs: 30,
                queue_ttl_secs: 30,
            },
            100,
        )
        .unwrap();
    let url = format!("ws://{address}/v1/events/ws?after=0");
    assert!(tokio_tungstenite::connect_async(&url).await.is_err());
    let mut request = url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert("Authorization", format!("Bearer {token}").parse().unwrap());
    let (mut socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();
    let message = tokio::time::timeout(Duration::from_secs(3), socket.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let first: Event = serde_json::from_str(message.to_text().unwrap()).unwrap();
    assert_eq!(first.task_id, task.id);
    socket.close(None).await.unwrap();
    store.cancel(&task.id, 101).unwrap();
    let mut request = format!("ws://{address}/v1/events/ws?after={}", first.sequence)
        .into_client_request()
        .unwrap();
    request
        .headers_mut()
        .insert("Authorization", format!("Bearer {token}").parse().unwrap());
    let (mut socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();
    let message = tokio::time::timeout(Duration::from_secs(3), socket.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let second: Event = serde_json::from_str(message.to_text().unwrap()).unwrap();
    assert_eq!(second.status, citta::model::Status::Cancelled);
    assert!(second.sequence > first.sequence);
    socket.close(None).await.unwrap();
    stop.cancel();
    tokio::time::timeout(Duration::from_secs(3), server)
        .await
        .unwrap()
        .unwrap();
}
