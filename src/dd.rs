//! TALKING TO A DATADUNGEON.
//!
//! Everything the console shows comes from a store's JSON API, and almost all of it is passed through
//! untouched: curio does not reshape DataDungeon's answers, so a field DataDungeon adds tomorrow shows
//! up without curio learning about it. What curio adds is the token, the URL, and an error a person
//! can act on when the store does not answer.
//!
//! WHICH TOKEN IS DECIDED HERE, AND ONLY HERE. A read sends the read token, or nothing for a store
//! that needs none. A write sends the write token. The write token would read too — DataDungeon
//! accepts it for both — but sending it on every read would put the stronger credential on every
//! request for no reason, so it goes only where it is needed.

use crate::config::Store;
use axum::body::{Body, Bytes};
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use reqwest::Url;
use serde_json::json;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Access {
    Read,
    Write,
}

/// What a write sends: JSON, as almost every write does, or a form, as an upload does. A form's
/// Content-Type carries the boundary its parts are split on, so it goes to the store as it came.
pub enum Payload {
    Json(Bytes),
    Form(Bytes, HeaderValue),
}

pub struct Dd {
    http: reqwest::Client,
}

impl Default for Dd {
    fn default() -> Self {
        Dd::new()
    }
}

impl Dd {
    pub fn new() -> Self {
        // A connect timeout but no overall one: an object can be a video, and streaming it to a
        // slow browser is not a hung store.
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .read_timeout(Duration::from_secs(60))
            .build()
            .expect("an HTTP client with default TLS");
        Dd { http }
    }

    /// The store's base URL plus `segments`, each escaped on its own. Type and tag names come from
    /// the store and are put back into its URLs, so a name with a slash or a space in it must stay
    /// one segment. A base with a path (an API mounted at `/dd`) keeps it.
    pub fn url(store: &Store, segments: &[&str], query: Option<&str>) -> Url {
        let mut url = Url::parse(&store.url).expect("validated when the config was read");
        url.path_segments_mut()
            .expect("an http(s) url has a path")
            .pop_if_empty()
            .extend(segments);
        url.set_query(query.filter(|q| !q.is_empty()));
        url
    }

    pub async fn send(
        &self,
        store: &Store,
        access: Access,
        method: Method,
        segments: &[&str],
        query: Option<&str>,
        body: Option<Payload>,
    ) -> Result<reqwest::Response, Response> {
        let url = Dd::url(store, segments, query);
        let mut req = self.http.request(method, url);
        let token = match access {
            Access::Read => store.read_token.as_deref(),
            Access::Write => store.write_token.as_deref(),
        };
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        match body {
            Some(Payload::Json(b)) => req = req.header(header::CONTENT_TYPE, "application/json").body(b),
            Some(Payload::Form(b, kind)) => req = req.header(header::CONTENT_TYPE, kind).body(b),
            None => {}
        }
        req.send().await.map_err(|e| {
            let why = if e.is_connect() { "did not answer" } else if e.is_timeout() { "timed out" } else { "failed" };
            error(StatusCode::BAD_GATEWAY,
                format!("{}: the DataDungeon at {} {why} ({})", store.name, store.url, root_cause(&e)))
        })
    }

    /// A read or write whose answer goes straight back to the browser.
    pub async fn relay(
        &self,
        store: &Store,
        access: Access,
        method: Method,
        segments: &[&str],
        query: Option<&str>,
        body: Option<Payload>,
    ) -> Response {
        match self.send(store, access, method, segments, query, body).await {
            Ok(res) => relay(res, store),
            Err(e) => e,
        }
    }
}

/// DataDungeon's status and body, as they came, streamed. Only the headers that describe the body
/// are carried over.
///
/// A 401 or 403 from the store is about curio's configured token, never about the person using the
/// console, so it is turned into a 502 that says so. Passed through as-is it would look like the
/// console asking the browser for credentials it does not have.
pub fn relay(res: reqwest::Response, store: &Store) -> Response {
    let status = res.status();
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return error(StatusCode::BAD_GATEWAY, format!(
            "{}: the store refused curio's token ({status}) — check read_token/write_token in curio's config",
            store.name));
    }
    let mut out = Response::builder().status(status);
    for h in [header::CONTENT_TYPE, header::CACHE_CONTROL, header::CONTENT_LENGTH] {
        if let Some(v) = res.headers().get(&h) {
            out = out.header(h, v.clone());
        }
    }
    out.body(Body::from_stream(res.bytes_stream()))
        .unwrap_or_else(|_| error(StatusCode::BAD_GATEWAY, "unreadable answer".into()))
}

/// Every error curio makes has DataDungeon's shape, `{"error": "…"}`, so the console reads one kind.
pub fn error(status: StatusCode, message: String) -> Response {
    let mut res = (status, axum::Json(json!({ "error": message }))).into_response();
    res.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

/// reqwest's own message is "error sending request for url (…)"; the useful part is at the bottom.
fn root_cause(e: &(dyn std::error::Error + 'static)) -> String {
    let mut cur = e;
    while let Some(next) = cur.source() {
        cur = next;
    }
    cur.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(url: &str) -> Store {
        Store { name: "t".into(), url: url.into(), read_token: None, write_token: None }
    }

    #[test]
    fn segments_are_escaped_one_at_a_time() {
        let u = Dd::url(&store("http://127.0.0.1:7077"), &["types", "a/b c", "tags"], None);
        assert_eq!(u.as_str(), "http://127.0.0.1:7077/types/a%2Fb%20c/tags",
            "a slash inside a name must not become a path separator");
    }

    #[test]
    fn a_mounted_api_keeps_its_prefix() {
        for base in ["https://cattacula.example/dd", "https://cattacula.example/dd/"] {
            let u = Dd::url(&store(base), &["assets", "7"], None);
            assert_eq!(u.as_str(), "https://cattacula.example/dd/assets/7", "{base}");
        }
    }

    #[test]
    fn the_query_passes_through_as_given() {
        let u = Dd::url(&store("http://h"), &["assets"], Some("type=voice&tag.voice=onyx&limit=50"));
        assert_eq!(u.as_str(), "http://h/assets?type=voice&tag.voice=onyx&limit=50");
        assert_eq!(Dd::url(&store("http://h"), &["types"], Some("")).as_str(), "http://h/types");
    }
}
