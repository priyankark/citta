use super::*;
use crate::{
    api::{self, App},
    store::Store,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        self,
        pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer},
    },
};

const CA: &[u8] = include_bytes!("../../tests/fixtures/enrollment-test-ca.der");
const CERT: &[u8] = include_bytes!("../../tests/fixtures/enrollment-test-cert.der");
const KEY: &[u8] = include_bytes!("../../tests/fixtures/enrollment-test-key.der");
fn device(id: &str, user: u64, dns: &str) -> Device {
    Device {
        id: id.into(),
        user_id: user,
        dns_name: dns.into(),
        ips: vec!["100.64.0.1".parse().unwrap()],
        tags: None,
        expired: false,
        online: true,
    }
}
fn settings() -> Settings {
    Settings {
        tailscale_bin: PathBuf::from("/bin/true"),
        owner_user_id: 7,
        tailnet_id: "tail-test".into(),
        device_id: "alpha".into(),
        https_port: 7332,
    }
}
fn status_fixture() -> Status {
    Status {
        backend_state: "Running".into(),
        this: device("alpha", 7, "alpha.tail-test.ts.net."),
        current_tailnet: Tailnet {
            stable_id: "tail-test".into(),
            suffix: "tail-test.ts.net".into(),
        },
        peer: BTreeMap::from([("key".into(), device("beta", 7, "beta.tail-test.ts.net."))]),
    }
}
#[test]
fn only_current_untagged_same_owner_netmap_devices_are_eligible() {
    let settings = settings();
    let mut status = status_fixture();
    assert_eq!(candidates(&status, &settings).unwrap().len(), 1);
    status.peer.get_mut("key").unwrap().user_id = 8;
    assert!(candidates(&status, &settings).unwrap().is_empty());
    status.peer.get_mut("key").unwrap().user_id = 7;
    status.peer.get_mut("key").unwrap().tags = Some(vec!["tag:server".into()]);
    assert!(candidates(&status, &settings).unwrap().is_empty());
    status.peer.get_mut("key").unwrap().tags = None;
    status.peer.get_mut("key").unwrap().expired = true;
    assert!(candidates(&status, &settings).unwrap().is_empty());
    status.this.user_id = 8;
    assert!(candidates(&status, &settings).is_err());
    status = status_fixture();
    status.current_tailnet.stable_id = "different".into();
    assert!(candidates(&status, &settings).is_err());
    status = status_fixture();
    status.this.id = "reinstalled".into();
    assert!(candidates(&status, &settings).is_err());
    status = status_fixture();
    status.backend_state = "Stopped".into();
    assert!(candidates(&status, &settings).is_err());
}
#[test]
fn untrusted_urls_and_non_tailscale_addresses_never_become_callbacks() {
    let s = settings();
    for dns in [
        "evil.example.com",
        "alpha.tail-test.ts.net.evil.com",
        "u:p@alpha.tail-test.ts.net",
        "alpha.tail-test.ts.net/path",
        ".tail-test.ts.net",
    ] {
        assert!(
            candidate(&device("peer", 7, dns), "tail-test.ts.net", &s).is_err(),
            "{dns}"
        );
    }
    let mut d = device("peer", 7, "beta.tail-test.ts.net");
    d.ips = vec!["127.0.0.1".parse().unwrap(), "192.168.1.1".parse().unwrap()];
    assert!(candidate(&d, "tail-test.ts.net", &s).is_err());
    for ip in ["100.64.0.1", "100.127.255.254", "fd7a:115c:a1e0::1"] {
        assert!(tail_address(ip.parse().unwrap()));
    }
    for ip in ["100.63.255.255", "100.128.0.1", "::1", "8.8.8.8"] {
        assert!(!tail_address(ip.parse().unwrap()));
    }
}
#[test]
fn existing_serve_routes_and_public_funnel_are_preserved_not_overwritten() {
    let url = "https://alpha.tail-test.ts.net:7332";
    let backend = "http://127.0.0.1:7331";
    assert!(ensure_serve(&json!({}), url, backend, 7332).is_ok());
    let good = json!({"TCP":{"7332":{"HTTPS":true}},"Web":{"alpha.tail-test.ts.net:7332":{"Handlers":{"/":{"Proxy":backend}}}}});
    assert!(ensure_serve(&good, url, backend, 7332).is_ok());
    assert!(ensure_serve(&good, url, "http://127.0.0.1:9999", 7332).is_err());
    let mut public = good.clone();
    public["AllowFunnel"] = json!({"alpha.tail-test.ts.net:7332":true});
    assert!(ensure_serve(&public, url, backend, 7332).is_err());
    assert!(
        ensure_serve(
            &json!({"TCP":{"7332":{"TCPForward":"x"}}}),
            url,
            backend,
            7332
        )
        .is_err()
    );
}

struct Listener {
    tcp: TcpListener,
    tls: TlsAcceptor,
}
impl axum::serve::Listener for Listener {
    type Io = tokio_rustls::server::TlsStream<TcpStream>;
    type Addr = SocketAddr;
    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            let (tcp, addr) = self.tcp.accept().await.unwrap();
            if let Ok(tls) = self.tls.accept(tcp).await {
                return (tls, addr);
            }
        }
    }
    fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.tcp.local_addr()
    }
}
async fn listener() -> Listener {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let tls = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(CERT.to_vec())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(KEY.to_vec())),
        )
        .unwrap();
    Listener {
        tcp: TcpListener::bind("127.0.0.1:0").await.unwrap(),
        tls: TlsAcceptor::from(Arc::new(tls)),
    }
}
fn setup(dir: &Path, id: &str) -> Arc<Enrollment> {
    let path = config::init(dir, id.into(), 7331).unwrap();
    let mut cfg = Config::load(&path).unwrap();
    let mut set = settings();
    set.device_id = id.into();
    cfg.enrollment = Some(set);
    Enrollment::open(Arc::new(cfg)).unwrap().unwrap()
}
fn remote(id: &str, port: u16) -> Candidate {
    Candidate {
        id: id.into(),
        url: format!("https://{id}.tail-test.ts.net:{port}"),
        address: "127.0.0.1".parse().unwrap(),
        online: true,
        test_ca: Some(CA.to_vec()),
    }
}
fn trust(enrollment: &Enrollment, remote: Candidate) {
    let mut inner = enrollment.inner.lock().unwrap();
    inner.refreshed = Some(Instant::now());
    inner.candidates.insert(remote.id.clone(), remote);
}
fn pending(enrollment: &Enrollment, remote: Candidate) -> String {
    let nonce = secret();
    enrollment.inner.lock().unwrap().pending.insert(
        remote.id.clone(),
        Pending {
            challenge: nonce.clone(),
            candidate: remote,
            at: Instant::now(),
        },
    );
    nonce
}
#[tokio::test]
async fn verified_https_handshake_pairs_without_admin_tokens_and_survives_restart() {
    let a_dir = tempfile::tempdir().unwrap();
    let b_dir = tempfile::tempdir().unwrap();
    let a = setup(a_dir.path(), "alpha");
    let b = setup(b_dir.path(), "beta");
    let al = listener().await;
    let bl = listener().await;
    let ar = remote("alpha", al.tcp.local_addr().unwrap().port());
    let br = remote("beta", bl.tcp.local_addr().unwrap().port());
    trust(&a, br.clone());
    trust(&b, ar.clone());
    let a_app = router(a.clone());
    let b_app = router(b.clone());
    let a_task = tokio::spawn(async move { axum::serve(al, a_app).await.unwrap() });
    let b_task = tokio::spawn(async move { axum::serve(bl, b_app).await.unwrap() });
    a.pair(br.clone()).await.unwrap();
    let (a_in, a_out) = {
        let i = a.inner.lock().unwrap();
        let r = &i.records["beta"];
        (r.inbound.clone(), r.outbound.clone())
    };
    assert_eq!(a.authenticate(&a_in), Some("beta".into()));
    assert_eq!(b.authenticate(&a_out), Some("alpha".into()));
    assert_ne!(a_in, config::read_token(&a.config.token_file).unwrap());
    assert_ne!(a_out, config::read_token(&b.config.token_file).unwrap());
    assert_eq!(a.clients()[0].node_id, "beta");
    let restored = Enrollment::open(a.config.clone()).unwrap().unwrap();
    assert!(
        restored.authenticate(&a_in).is_none(),
        "disk alone must not establish current trust"
    );
    trust(&restored, br.clone());
    assert_eq!(restored.authenticate(&a_in), Some("beta".into()));
    let report = restored.report().to_string();
    assert!(!report.contains(&a_in));
    assert!(!report.contains(&a_out));
    // Real API authentication accepts only the peer's identity for claims.
    let app = App {
        config: b.config.clone(),
        store: Arc::new(Store::open(&b.config.data_dir, "beta", 30, 1000, 10000).unwrap()),
        token: Arc::new(config::read_token(&b.config.token_file).unwrap()),
        peers: vec![],
        enrollment: Some(b.clone()),
        inventory: Default::default(),
        managed_agents: Default::default(),
        panel: Default::default(),
        activity: Arc::new(Mutex::new(BTreeMap::new())),
        shutdown: CancellationToken::new(),
    };
    let tcp = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = tcp.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(tcp, api::router(app)).await.unwrap() });
    let client = Client::new(&format!("http://{addr}"), a_out, "alpha".into()).unwrap();
    let _: Value = client.get("/v1/node").await.unwrap();
    assert!(
        client
            .post::<Value>("/v1/panel/session", &json!({}))
            .await
            .is_err()
    );
    assert!(
        client
            .post::<Value>(
                "/v1/claim",
                &json!({"worker_node":"beta","capability":"echo"})
            )
            .await
            .is_err()
    );
    let _: Value = client
        .post(
            "/v1/claim",
            &json!({"worker_node":"alpha","capability":"echo"}),
        )
        .await
        .unwrap();
    b.inner.lock().unwrap().candidates.clear();
    assert!(client.get::<Value>("/v1/node").await.is_err());
    a_task.abort();
    b_task.abort();
    server.abort();
}
#[tokio::test]
async fn forged_expired_replayed_and_browser_confirmations_cannot_pair() {
    let dir = tempfile::tempdir().unwrap();
    let a = setup(dir.path(), "alpha");
    let b = remote("beta", 7332);
    trust(&a, b.clone());
    let good = pending(&a, b.clone());
    let request = |nonce: String| Confirm {
        device_id: "beta".into(),
        challenge: nonce,
        node_id: "beta".into(),
        token: secret(),
    };
    assert!(matches!(
        confirm(State(a.clone()), HeaderMap::new(), Json(request(secret()))).await,
        Err(StatusCode::FORBIDDEN)
    ));
    assert!(a.clients().is_empty());
    let mut headers = HeaderMap::new();
    headers.insert("origin", "https://evil.example".parse().unwrap());
    assert!(matches!(
        confirm(State(a.clone()), headers, Json(request(good.clone()))).await,
        Err(StatusCode::FORBIDDEN)
    ));
    let _ = confirm(
        State(a.clone()),
        HeaderMap::new(),
        Json(request(good.clone())),
    )
    .await
    .unwrap();
    assert!(matches!(
        confirm(State(a.clone()), HeaderMap::new(), Json(request(good))).await,
        Err(StatusCode::FORBIDDEN)
    ));
    let expired = pending(&a, b.clone());
    a.inner.lock().unwrap().pending.get_mut("beta").unwrap().at = Instant::now() - FRESH;
    assert!(matches!(
        confirm(State(a.clone()), HeaderMap::new(), Json(request(expired))).await,
        Err(StatusCode::FORBIDDEN)
    ));
    let fresh_nonce = pending(&a, b);
    a.inner.lock().unwrap().refreshed = Some(Instant::now() - FRESH);
    assert!(
        confirm(
            State(a.clone()),
            HeaderMap::new(),
            Json(request(fresh_nonce))
        )
        .await
        .is_err()
    );
    assert!(a.clients().is_empty());
}
#[tokio::test]
async fn caller_cannot_choose_a_callback_url_or_receive_a_token_from_start() {
    let dir = tempfile::tempdir().unwrap();
    let a = setup(dir.path(), "alpha");
    a.inner.lock().unwrap().refreshed = Some(Instant::now());
    assert!(
        serde_json::from_value::<Start>(
            json!({"device_id":"beta","challenge":secret(),"url":"https://evil.example"})
        )
        .is_err()
    );
    let result = start(
        State(a),
        HeaderMap::new(),
        Json(Start {
            device_id: "foreign".into(),
            challenge: secret(),
        }),
    )
    .await;
    assert!(matches!(result, Err(StatusCode::FORBIDDEN)));
}
#[tokio::test]
async fn tls_certificate_validation_and_redirect_refusal_are_enforced() {
    let tls = listener().await;
    let port = tls.tcp.local_addr().unwrap().port();
    let task = tokio::spawn(async move {
        axum::serve(
            tls,
            Router::new().route(
                "/test",
                post(|| async { axum::response::Redirect::temporary("http://127.0.0.1:1/leak") }),
            ),
        )
        .await
        .unwrap()
    });
    let r = remote("beta", port);
    assert!(exchange::<Value>(&r, "/test", &json!({})).await.is_err());
    let mut untrusted = r.clone();
    untrusted.test_ca = None;
    assert!(
        exchange::<Value>(&untrusted, "/test", &json!({}))
            .await
            .is_err()
    );
    let mut wrong_host = r.clone();
    wrong_host.url = format!("https://wrong.tail-test.ts.net:{port}");
    assert!(
        exchange::<Value>(&wrong_host, "/test", &json!({}))
            .await
            .is_err()
    );
    let mut plain = r;
    plain.url = format!("http://127.0.0.1:{port}");
    assert!(
        exchange::<Value>(&plain, "/test", &json!({}))
            .await
            .is_err()
    );
    task.abort();
}
#[tokio::test]
async fn netmap_removal_revokes_credentials_durably_and_cli_failure_fails_closed() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let original = setup(dir.path(), "alpha");
    let bin = dir.path().join("tailscale-fixture");
    std::fs::write(
        &bin,
        "#!/bin/sh\n[ \"$TAILSCALE_BE_CLI\" = 1 ] || exit 23\nexec /bin/cat \"$0.json\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o700)).unwrap();
    let status_path = dir.path().join("tailscale-fixture.json");
    let mut data = json!({"BackendState":"Running","Self":{"ID":"alpha","UserID":7,"DNSName":"alpha.tail-test.ts.net","TailscaleIPs":["100.64.0.1"]},"CurrentTailnet":{"StableID":"tail-test","MagicDNSSuffix":"tail-test.ts.net"},"Peer":{"b":{"ID":"beta","UserID":7,"DNSName":"beta.tail-test.ts.net","TailscaleIPs":["100.64.0.2"],"Online":true}}});
    std::fs::write(&status_path, data.to_string()).unwrap();
    let mut config = (*original.config).clone();
    config.enrollment.as_mut().unwrap().tailscale_bin = bin;
    let a = Enrollment::open(Arc::new(config)).unwrap().unwrap();
    a.refresh().await.unwrap();
    let credential = secret();
    {
        let mut inner = a.inner.lock().unwrap();
        let candidate = inner.candidates["beta"].clone();
        a.commit(
            &mut inner,
            &candidate,
            "beta".into(),
            credential.clone(),
            secret(),
        )
        .unwrap();
    }
    assert_eq!(a.authenticate(&credential), Some("beta".into()));
    std::fs::write(&status_path, "not JSON").unwrap();
    assert!(a.refresh().await.is_err());
    assert!(a.authenticate(&credential).is_none());
    assert!(a.clients().is_empty());
    data["Peer"] = json!({});
    std::fs::write(&status_path, data.to_string()).unwrap();
    a.refresh().await.unwrap();
    assert!(a.authenticate(&credential).is_none());
    let restored = Enrollment::open(a.config.clone()).unwrap().unwrap();
    assert!(restored.inner.lock().unwrap().records.is_empty());
}

#[tokio::test]
async fn duplicate_node_ids_cannot_replace_another_device() {
    let dir = tempfile::tempdir().unwrap();
    let a = setup(dir.path(), "alpha");
    let b = remote("beta", 7332);
    let c = remote("gamma", 7332);
    trust(&a, b.clone());
    trust(&a, c.clone());
    let mut inner = a.inner.lock().unwrap();
    a.commit(&mut inner, &b, "worker".into(), secret(), secret())
        .unwrap();
    assert!(
        a.commit(&mut inner, &c, "worker".into(), secret(), secret())
            .is_err()
    );
    assert_eq!(inner.records.len(), 1);
}
