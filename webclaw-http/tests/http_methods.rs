//! Test all HTTP methods, cookies, redirects, error handling, and stress.

use webclaw_http::{Client, BandwidthStats};

fn build_client() -> Client {
    Client::builder()
        .chrome()
        .build()
        .expect("client should build")
}

// --- HTTP Methods ---

#[tokio::test]
async fn get_returns_200() {
    let client = build_client();
    let resp = client.get("https://httpbin.org/get").await.expect("GET");
    assert_eq!(resp.status(), 200);
    assert!(resp.body().len() > 50);
}

#[tokio::test]
async fn post_sends_body() {
    let client = build_client();
    let body = b"hello=world";
    let resp = client.post("https://httpbin.org/post", body).await.expect("POST");
    assert_eq!(resp.status(), 200);
    let text = resp.into_text();
    assert!(text.contains("hello=world"), "POST body not echoed: {}", &text[..200.min(text.len())]);
}

#[tokio::test]
async fn put_sends_body() {
    let client = build_client();
    let resp = client.put("https://httpbin.org/put", b"data=123").await.expect("PUT");
    assert_eq!(resp.status(), 200);
    let text = resp.into_text();
    assert!(text.contains("data=123"));
}

#[tokio::test]
async fn delete_works() {
    let client = build_client();
    let resp = client.delete("https://httpbin.org/delete").await.expect("DELETE");
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn patch_sends_body() {
    let client = build_client();
    let resp = client.patch("https://httpbin.org/patch", b"patch=yes").await.expect("PATCH");
    assert_eq!(resp.status(), 200);
    let text = resp.into_text();
    assert!(text.contains("patch=yes"));
}

#[tokio::test]
async fn head_returns_no_body() {
    let client = build_client();
    let resp = client.head("https://httpbin.org/get").await.expect("HEAD");
    assert_eq!(resp.status(), 200);
    assert!(resp.body().is_empty(), "HEAD should have empty body");
}

// --- Cookies ---

#[tokio::test]
async fn cookies_persist_across_requests() {
    let client = build_client();

    // First request: server sets a cookie
    let resp = client
        .get("https://httpbin.org/cookies/set/testcookie/webclaw123")
        .await
        .expect("set cookie");
    // httpbin redirects to /cookies after setting
    assert!(resp.is_success());

    // Second request: cookie should be sent back
    let resp = client.get("https://httpbin.org/cookies").await.expect("get cookies");
    let text = resp.into_text();
    assert!(
        text.contains("webclaw123"),
        "cookie not persisted: {}", &text[..200.min(text.len())]
    );
}

#[tokio::test]
async fn multiple_cookies_tracked() {
    let client = build_client();

    client.get("https://httpbin.org/cookies/set/a/1").await.ok();
    client.get("https://httpbin.org/cookies/set/b/2").await.ok();

    let resp = client.get("https://httpbin.org/cookies").await.expect("cookies");
    let text = resp.into_text();
    assert!(text.contains("\"a\": \"1\"") || text.contains("\"a\":\"1\""), "cookie a missing");
    assert!(text.contains("\"b\": \"2\"") || text.contains("\"b\":\"2\""), "cookie b missing");
}

// --- Redirects ---

#[tokio::test]
async fn follows_redirects() {
    let client = build_client();
    // httpbin redirects to /get
    let resp = client
        .get("https://httpbin.org/redirect-to?url=https%3A%2F%2Fhttpbin.org%2Fget&status_code=302")
        .await
        .expect("redirect");
    assert_eq!(resp.status(), 200);
    assert!(resp.url().contains("/get"), "should follow redirect to /get, got {}", resp.url());
}

// --- Error Handling ---

#[tokio::test]
async fn invalid_url_returns_error() {
    let client = build_client();
    let result = client.get("not-a-url").await;
    assert!(result.is_err(), "invalid URL should error");
}

#[tokio::test]
async fn timeout_returns_error() {
    let client = Client::builder()
        .chrome()
        .timeout(std::time::Duration::from_millis(1)) // 1ms timeout
        .build()
        .expect("build");

    let result = client.get("https://httpbin.org/delay/5").await;
    assert!(result.is_err(), "should timeout");
}

#[tokio::test]
async fn dns_failure_returns_error() {
    let client = build_client();
    let result = client.get("https://this-domain-does-not-exist-xyz123.com").await;
    assert!(result.is_err(), "nonexistent domain should error");
}

#[tokio::test]
async fn handles_404() {
    let client = build_client();
    let resp = client.get("https://httpbin.org/status/404").await.expect("404");
    assert_eq!(resp.status(), 404);
    assert!(!resp.is_success());
}

#[tokio::test]
async fn handles_500() {
    let client = build_client();
    let resp = client.get("https://httpbin.org/status/500").await.expect("500");
    assert_eq!(resp.status(), 500);
    assert!(!resp.is_success());
}

// --- Response Parsing ---

#[tokio::test]
async fn content_type_header() {
    let client = build_client();
    let resp = client.get("https://httpbin.org/json").await.expect("json");
    assert!(
        resp.content_type().unwrap_or("").contains("application/json"),
        "expected JSON content type"
    );
}

#[tokio::test]
async fn response_header_lookup_case_insensitive() {
    let client = build_client();
    let resp = client.get("https://httpbin.org/get").await.expect("get");
    // httpbin returns Content-Type header
    let ct1 = resp.header("content-type");
    let ct2 = resp.header("Content-Type");
    assert_eq!(ct1, ct2, "header lookup should be case insensitive");
}

// --- Bandwidth Tracking ---

#[tokio::test]
async fn bandwidth_tracks_across_requests() {
    let stats = BandwidthStats::new();
    let client = Client::builder()
        .chrome()
        .bandwidth_tracker(stats.clone())
        .build()
        .expect("build");

    client.get("https://httpbin.org/get").await.ok();
    client.get("https://httpbin.org/json").await.ok();

    assert_eq!(stats.request_count(), 2, "should track 2 requests");
    assert!(stats.total_received() > 0, "should have received bytes");
    assert!(stats.total_sent() > 0, "should have sent bytes");
}

#[tokio::test]
async fn bandwidth_shared_across_clones() {
    let stats = BandwidthStats::new();
    let client = Client::builder()
        .chrome()
        .bandwidth_tracker(stats.clone())
        .build()
        .expect("build");

    let client2 = client.clone();

    client.get("https://httpbin.org/get").await.ok();
    client2.get("https://httpbin.org/json").await.ok();

    assert_eq!(stats.request_count(), 2, "clones should share bandwidth tracker");
}

// --- Stress Test ---

#[tokio::test]
async fn concurrent_requests() {
    let client = build_client();
    let mut handles = Vec::new();

    for i in 0..10 {
        let c = client.clone();
        handles.push(tokio::spawn(async move {
            let url = format!("https://httpbin.org/get?id={i}");
            c.get(&url).await.map(|r| r.status())
        }));
    }

    let mut successes = 0;
    for handle in handles {
        if let Ok(Ok(status)) = handle.await {
            if status == 200 {
                successes += 1;
            }
        }
    }

    assert!(successes >= 8, "at least 8/10 concurrent requests should succeed, got {successes}");
}
