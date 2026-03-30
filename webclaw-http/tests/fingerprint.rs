//! Integration test: verify TLS fingerprint matches real browsers.
//!
//! Hits tls.peet.ws/api/all and checks JA4 against known real browser values.
//! Requires network access — skip in CI with `--skip fingerprint`.

use webclaw_http::Client;

const CHROME_JA4: &str = "t13d1517h2_8daaf6152771_b6f405a00624";

#[tokio::test]
async fn chrome_ja4_matches_real_browser() {
    let client = Client::builder()
        .chrome()
        .build()
        .expect("client should build");

    let resp = client
        .get("https://tls.peet.ws/api/all")
        .await
        .expect("request should succeed");

    assert!(resp.is_success(), "status {}", resp.status());

    let body = resp.text();
    let fp: serde_json::Value = serde_json::from_str(&body).expect("response should be JSON");

    let ja4 = fp["tls"]["ja4"].as_str().expect("JA4 should be present");

    // JA4 segments: version+count, cipher_hash, extension_hash
    let our_parts: Vec<&str> = ja4.split('_').collect();
    let target_parts: Vec<&str> = CHROME_JA4.split('_').collect();

    assert_eq!(our_parts.len(), 3, "JA4 should have 3 segments, got: {ja4}");

    // Cipher hash must match (same cipher suites)
    assert_eq!(
        our_parts[1], target_parts[1],
        "cipher hash mismatch: got {}, expected {}",
        our_parts[1], target_parts[1]
    );

    // Extension hash must match (same extension set)
    assert_eq!(
        our_parts[2], target_parts[2],
        "extension hash mismatch: got {}, expected {}",
        our_parts[2], target_parts[2]
    );

    // Full JA4 match
    assert_eq!(ja4, CHROME_JA4, "JA4 mismatch: got {ja4}");

    // Verify extension count
    let ext_count = fp["tls"]["extensions"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    assert!(ext_count >= 18, "expected >=18 extensions, got {ext_count}");

    // Verify HTTP/2 Akamai fingerprint
    let akamai = fp["http2"]["akamai_fingerprint_hash"]
        .as_str()
        .unwrap_or("");
    assert_eq!(
        akamai, "52d84b11737d980aef856699f885ca86",
        "HTTP/2 Akamai hash mismatch"
    );

    // Bandwidth tracking should have recorded something
    assert!(client.bandwidth().total_received() > 0);
    assert!(client.bandwidth().request_count() == 1);
}

#[tokio::test]
async fn firefox_cipher_and_extension_hash_match() {
    let client = Client::builder()
        .firefox()
        .build()
        .expect("client should build");

    let resp = client
        .get("https://tls.peet.ws/api/all")
        .await
        .expect("request should succeed");

    let body = resp.text();
    let fp: serde_json::Value = serde_json::from_str(&body).expect("response should be JSON");

    let ja4 = fp["tls"]["ja4"].as_str().expect("JA4 should be present");
    let parts: Vec<&str> = ja4.split('_').collect();

    // Firefox cipher hash
    assert_eq!(parts[1], "5b57614c22b0", "Firefox cipher hash mismatch");
    // Firefox extension hash
    assert_eq!(parts[2], "3cbfd9057e0d", "Firefox extension hash mismatch");
}

#[tokio::test]
async fn safari_has_distinct_fingerprint() {
    let client = Client::builder()
        .safari()
        .build()
        .expect("client should build");

    let resp = client
        .get("https://tls.peet.ws/api/all")
        .await
        .expect("request should succeed");

    let body = resp.text();
    let fp: serde_json::Value = serde_json::from_str(&body).expect("response should be JSON");

    let ja4 = fp["tls"]["ja4"].as_str().expect("JA4 should be present");
    let parts: Vec<&str> = ja4.split('_').collect();
    assert_eq!(parts.len(), 3, "JA4 should have 3 segments, got: {ja4}");

    // Safari must differ from Chrome — if they match, our profile isn't working
    let chrome_cipher = "8daaf6152771";
    assert_ne!(
        parts[1], chrome_cipher,
        "Safari cipher hash should differ from Chrome"
    );

    // Verify HTTP/2 Akamai fingerprint exists and differs from Chrome
    let akamai = fp["http2"]["akamai_fingerprint_hash"]
        .as_str()
        .unwrap_or("");
    assert!(
        !akamai.is_empty(),
        "Safari should produce an Akamai fingerprint"
    );
    assert_ne!(
        akamai, "52d84b11737d980aef856699f885ca86",
        "Safari Akamai hash should differ from Chrome"
    );
}

#[tokio::test]
async fn edge_matches_chrome_fingerprint() {
    let client = Client::builder()
        .edge()
        .build()
        .expect("client should build");

    let resp = client
        .get("https://tls.peet.ws/api/all")
        .await
        .expect("request should succeed");

    let body = resp.text();
    let fp: serde_json::Value = serde_json::from_str(&body).expect("response should be JSON");

    let ja4 = fp["tls"]["ja4"].as_str().expect("JA4 should be present");

    // Edge is Chromium-based — JA4 should match Chrome exactly
    assert_eq!(ja4, CHROME_JA4, "Edge JA4 should match Chrome: got {ja4}");
}
