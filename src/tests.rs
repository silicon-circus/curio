//! curio against a fake DataDungeon, over real sockets.
//!
//! The unit tests in each module check the pieces. These check the claims that only hold for the
//! whole: which token reaches the store on which request, that a write cannot get past a locked
//! switch, and that a thumbnail once made no longer needs the store at all.

use super::*;
use axum::extract::Request;
use axum::http::{header, Method, StatusCode};
use axum::response::IntoResponse;
use std::sync::Mutex;

/// What the fake store saw: method, path and query, and the token, if any.
type Seen = Arc<Mutex<Vec<(Method, String, Option<String>)>>>;

fn png() -> Vec<u8> {
    let img = image::RgbImage::from_fn(640, 320, |x, y| image::Rgb([x as u8, y as u8, 90]));
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(img).write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

fn sha(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

async fn fake_dd(seen: Seen) -> String {
    let handler = move |req: Request| {
        let seen = seen.clone();
        async move {
            let auth = req.headers().get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok()).map(|v| v.trim_start_matches("Bearer ").to_string());
            let at = req.uri().path_and_query().map(|p| p.to_string()).unwrap_or_default();
            seen.lock().unwrap().push((req.method().clone(), at.clone(), auth));
            let path = req.uri().path().to_string();
            if path.starts_with("/strict/") {
                return (StatusCode::UNAUTHORIZED, r#"{"error":"Reads need a token"}"#).into_response();
            }
            let json = |s: &'static str| ([(header::CONTENT_TYPE, "application/json")], s).into_response();
            match (req.method().as_str(), path.as_str()) {
                ("GET", "/types") => json(r#"[{"name":"art","count":2,"registration":null}]"#),
                ("GET", "/types/a%2Fb/tags") => json(r#"{"venue":2}"#),
                ("GET", "/assets") => json(r#"{"total":0,"limit":50,"offset":0,"assets":[]}"#),
                ("GET", "/assets/1/object") => ([(header::CONTENT_TYPE, "image/png")], png()).into_response(),
                ("GET", "/assets/2/object") =>
                    ([(header::CONTENT_TYPE, "image/svg+xml")], "<svg xmlns='http://www.w3.org/2000/svg'/>").into_response(),
                ("PATCH", "/assets/1") => json(r#"{"asset":{"id":1},"tags":{}}"#),
                ("DELETE", "/assets/1") => json(r#"{"deleted":1}"#),
                ("POST", "/maintenance/check") => json(r#"{"missing_objects":[],"orphan_objects":[]}"#),
                ("GET", "/assets/1/links") => json(r#"{"from":[],"to":[]}"#),
                ("POST", "/assets/1/retire") | ("POST", "/assets/1/reinstate") => json(r#"{"asset":{"id":1}}"#),
                ("POST", "/maintenance/prune") => json(r#"{"pruned":0}"#),
                ("POST", "/maintenance/measure") => json(r#"{"measured":0}"#),
                ("POST", "/assets/1/links") => json(r#"{"from":[]}"#),
                ("DELETE", "/assets/1/links") => json(r#"{"removed":true}"#),
                _ => (StatusCode::NOT_FOUND, r#"{"error":"No such endpoint"}"#).into_response(),
            }
        }
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, Router::new().fallback(handler)).await.unwrap() });
    url
}

struct Rig {
    base: String,
    seen: Seen,
    http: reqwest::Client,
    thumbs: PathBuf,
}

impl Rig {
    async fn new(name: &str) -> Rig {
        let seen: Seen = Arc::default();
        let dd = fake_dd(seen.clone()).await;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut cfg = Config::parse(&format!(r#"
            [[store]]
            name = "open"
            url = "{dd}"
            read_token = "READ"
            [[store]]
            name = "curated"
            url = "{dd}"
            read_token = "READ"
            write_token = "WRITE"
            [[store]]
            name = "strict"
            url = "{dd}/strict"
        "#)).unwrap();
        cfg.port = port;
        let thumbs = std::env::temp_dir().join(format!("curio-it-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&thumbs);
        cfg.thumbs = thumbs.clone();
        let st = state(cfg).unwrap();
        tokio::spawn(async move { axum::serve(listener, app(st)).await.unwrap() });
        Rig { base: format!("http://127.0.0.1:{port}"), seen, http: reqwest::Client::new(), thumbs }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    fn tokens_seen(&self) -> Vec<(Method, String, Option<String>)> {
        std::mem::take(&mut *self.seen.lock().unwrap())
    }

    async fn unlock(&self) -> String {
        let v: serde_json::Value = self.http.post(self.url("/api/update")).send().await.unwrap().json().await.unwrap();
        v["key"].as_str().unwrap().to_string()
    }
}

impl Drop for Rig {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.thumbs);
    }
}

#[tokio::test]
async fn reads_carry_the_read_token_and_never_the_write_token() {
    let r = Rig::new("reads").await;
    for path in ["/api/stores/curated/types", "/api/stores/curated/assets?type=art&tag.venue=boardwalk"] {
        let res = r.http.get(r.url(path)).send().await.unwrap();
        assert_eq!(res.status(), StatusCode::OK, "{path}");
    }
    let res = r.http.post(r.url("/api/stores/curated/check")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "the consistency check is a read");
    let seen = r.tokens_seen();
    assert_eq!(seen.len(), 3);
    for (m, at, token) in &seen {
        assert_eq!(token.as_deref(), Some("READ"), "{m} {at}");
    }
    assert_eq!(seen[1].1, "/assets?type=art&tag.venue=boardwalk", "the query reaches the store as written");
}

#[tokio::test]
async fn names_from_the_store_go_back_as_one_segment() {
    let r = Rig::new("segments").await;
    let res = r.http.get(r.url("/api/stores/open/types/a%2Fb/tags")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(r.tokens_seen()[0].1, "/types/a%2Fb/tags");
}

#[tokio::test]
async fn a_write_needs_a_write_token_and_the_switch_and_then_carries_the_write_token() {
    let r = Rig::new("writes").await;
    let patch = |store: &str, key: Option<&str>| {
        let mut req = r.http.patch(r.url(&format!("/api/stores/{store}/assets/1"))).body(r#"{"rating":3}"#);
        if let Some(k) = key {
            req = req.header("x-curio-update", k);
        }
        req.send()
    };

    assert_eq!(patch("curated", None).await.unwrap().status(), StatusCode::LOCKED, "switch off");
    assert_eq!(patch("curated", Some("guess")).await.unwrap().status(), StatusCode::LOCKED, "a made-up key");
    assert!(r.tokens_seen().is_empty(), "a refused write never reaches the store");

    let key = r.unlock().await;
    assert_eq!(patch("open", Some(&key)).await.unwrap().status(), StatusCode::FORBIDDEN,
        "the switch does not make a store without a write token writable");
    assert!(r.tokens_seen().is_empty());

    let res = patch("curated", Some(&key)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let promote = r.http.post(r.url("/api/stores/curated/types/art/promote"))
        .header("x-curio-update", &key).body(r#"{"trait":"venue","value":"pier"}"#).send().await.unwrap();
    let vacuum = r.http.post(r.url("/api/stores/curated/vacuum"))
        .header("x-curio-update", &key).body("{}").send().await.unwrap();
    assert_eq!((promote.status(), vacuum.status()), (StatusCode::NOT_FOUND, StatusCode::NOT_FOUND),
        "the fake store has neither; what matters is that they reached it, and how");
    let seen = r.tokens_seen();
    let write = Some("WRITE".to_string());
    assert_eq!(seen, vec![
        (Method::PATCH, "/assets/1".to_string(), write.clone()),
        (Method::POST, "/types/art/promote".to_string(), write.clone()),
        (Method::POST, "/maintenance/vacuum".to_string(), write),
    ]);

    let off = r.http.delete(r.url("/api/update")).send().await.unwrap();
    assert_eq!(off.status(), StatusCode::OK);
    let res = r.http.delete(r.url("/api/stores/curated/assets/1")).header("x-curio-update", &key).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::LOCKED, "turning it off ends the key");
}

#[tokio::test]
async fn links_are_read_with_the_read_token_and_changed_only_through_the_switch() {
    let r = Rig::new("links").await;
    let res = r.http.get(r.url("/api/stores/curated/assets/1/links")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let add = || r.http.post(r.url("/api/stores/curated/assets/1/links")).body(r#"{"rel":"used-by","name":"boardwalk"}"#);
    assert_eq!(add().send().await.unwrap().status(), StatusCode::LOCKED);
    let key = r.unlock().await;
    assert_eq!(add().header("x-curio-update", &key).send().await.unwrap().status(), StatusCode::OK);
    let res = r.http.delete(r.url("/api/stores/curated/assets/1/links?rel=used-by&name=boardwalk"))
        .header("x-curio-update", &key).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let read = Some("READ".to_string());
    let write = Some("WRITE".to_string());
    assert_eq!(r.tokens_seen(), vec![
        (Method::GET, "/assets/1/links".to_string(), read),
        (Method::POST, "/assets/1/links".to_string(), write.clone()),
        (Method::DELETE, "/assets/1/links?rel=used-by&name=boardwalk".to_string(), write),
    ], "an unlink's query reaches the store as written");
}

#[tokio::test]
async fn retiring_reinstating_and_pruning_are_writes_like_any_other() {
    let r = Rig::new("retire").await;
    let calls = [("/api/stores/curated/assets/1/retire", "/assets/1/retire"),
                 ("/api/stores/curated/assets/1/reinstate", "/assets/1/reinstate"),
                 ("/api/stores/curated/prune", "/maintenance/prune"),
                 ("/api/stores/curated/measure", "/maintenance/measure")];
    for (ours, _) in calls {
        let res = r.http.post(r.url(ours)).body("{}").send().await.unwrap();
        assert_eq!(res.status(), StatusCode::LOCKED, "{ours} with the switch off");
    }
    assert!(r.tokens_seen().is_empty());
    let key = r.unlock().await;
    for (ours, _) in calls {
        let res = r.http.post(r.url(ours)).header("x-curio-update", &key).body("{}").send().await.unwrap();
        assert_eq!(res.status(), StatusCode::OK, "{ours}");
    }
    let write = Some("WRITE".to_string());
    assert_eq!(r.tokens_seen(), calls.iter().map(|(_, theirs)| (Method::POST, theirs.to_string(), write.clone())).collect::<Vec<_>>());
}

#[tokio::test]
async fn a_store_refusing_curios_token_is_not_passed_off_as_the_users_problem() {
    let r = Rig::new("refused").await;
    let res = r.http.get(r.url("/api/stores/strict/types")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_GATEWAY);
    let v: serde_json::Value = res.json().await.unwrap();
    assert!(v["error"].as_str().unwrap().contains("curio's config"), "{v}");
}

#[tokio::test]
async fn requests_from_elsewhere_are_refused() {
    let r = Rig::new("gate").await;
    let res = r.http.get(r.url("/api/stores/open/types")).header("host", "rebound.example").send().await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN, "a rebinding page has a foreign Host");
    let res = r.http.get(r.url("/api/stores/open/types")).header("origin", "https://elsewhere.example").send().await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let key = r.http.post(r.url("/api/update")).header("origin", "https://elsewhere.example").send().await.unwrap();
    assert_eq!(key.status(), StatusCode::FORBIDDEN, "nor can another site turn the switch on");
    let local = r.http.post(r.url("/api/update")).header("origin", "http://localhost:3000").send().await.unwrap();
    assert_eq!(local.status(), StatusCode::FORBIDDEN, "nor a page from another port on this machine");
    for site in ["cross-site", "same-site"] {
        let img = r.http.get(r.url("/api/stores/open/assets/1/object")).header("sec-fetch-site", site).send().await.unwrap();
        assert_eq!(img.status(), StatusCode::FORBIDDEN, "an <img> on a {site} page");
    }
    assert!(r.tokens_seen().is_empty());
    for site in ["same-origin", "none"] {
        let ok = r.http.get(r.url("/api/stores/open/types")).header("sec-fetch-site", site).send().await.unwrap();
        assert_eq!(ok.status(), StatusCode::OK, "{site}: the console itself, or an address typed in");
    }
}

#[tokio::test]
async fn the_console_works_through_a_tunnel_on_another_port() {
    let r = Rig::new("tunnel").await;
    let res = r.http.post(r.url("/api/update"))
        .header("host", "localhost:8080").header("origin", "http://localhost:8080").send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "same-origin with the Host it was sent to");
}

#[tokio::test]
async fn objects_are_sandboxed_and_name_themselves_when_downloaded() {
    let r = Rig::new("object").await;
    let res = r.http.get(r.url("/api/stores/open/assets/2/object?download=1")).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let h = res.headers();
    assert_eq!(h[header::CONTENT_TYPE], "image/svg+xml");
    assert!(h[header::CONTENT_SECURITY_POLICY].to_str().unwrap().starts_with("sandbox"));
    assert_eq!(h[header::CONTENT_DISPOSITION], "attachment; filename=\"open-2.svg\"");
}

#[tokio::test]
async fn a_thumbnail_once_made_needs_the_store_no_more() {
    let r = Rig::new("thumb").await;
    let hash = sha(&png());
    let path = format!("/api/stores/open/assets/1/thumb/{hash}?w=300");
    let first = r.http.get(r.url(&path)).send().await.unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(first.headers()[header::CONTENT_TYPE], "image/webp");
    assert!(first.headers()[header::CACHE_CONTROL].to_str().unwrap().contains("immutable"));
    let img = image::load_from_memory(&first.bytes().await.unwrap()).unwrap();
    assert_eq!(img.width(), 320, "300 rounds up to 320");
    assert_eq!(r.tokens_seen().len(), 1);

    let again = r.http.get(r.url(&path)).send().await.unwrap();
    assert_eq!(again.status(), StatusCode::OK);
    assert!(r.tokens_seen().is_empty(), "served from the cache without asking the store");
}

#[tokio::test]
async fn a_thumbnail_is_not_filed_under_a_hash_its_bytes_do_not_have() {
    let r = Rig::new("lie").await;
    let wrong = "0".repeat(64);
    let res = r.http.get(r.url(&format!("/api/stores/open/assets/1/thumb/{wrong}"))).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK, "still made");
    assert_eq!(res.headers()[header::CACHE_CONTROL], "no-store");
    let res = r.http.get(r.url(&format!("/api/stores/open/assets/1/thumb/{wrong}"))).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(r.tokens_seen().len(), 2, "and made again, because it was never kept");

    let res = r.http.get(r.url(&format!("/api/stores/open/assets/2/thumb/{}", sha(b"x")))).send().await.unwrap();
    assert_eq!(res.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE, "an svg gets no thumbnail");
}

#[tokio::test]
async fn the_store_list_never_includes_a_token() {
    let r = Rig::new("list").await;
    let text = r.http.get(r.url("/api/stores")).send().await.unwrap().text().await.unwrap();
    assert!(!text.contains("READ") && !text.contains("WRITE"), "{text}");
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let writable: Vec<bool> = v["stores"].as_array().unwrap().iter().map(|s| s["writable"].as_bool().unwrap()).collect();
    assert_eq!(writable, [false, true, false]);
}
