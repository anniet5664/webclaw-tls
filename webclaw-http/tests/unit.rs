//! Hermetic unit tests — no network access required.
//!
//! Tests client builder validation, response methods, and error handling
//! without hitting external services.

use webclaw_http::{Client, Error};

// --- ClientBuilder validation ---

#[test]
fn build_without_profile_defaults_to_chrome() {
    let client = Client::builder().build().expect("default build");
    assert_eq!(client.profile_name(), "Chrome/146");
}

#[test]
fn build_with_each_profile() {
    for (name, client) in [
        ("Chrome/146", Client::builder().chrome().build()),
        ("Firefox/146", Client::builder().firefox().build()),
        ("Safari/18", Client::builder().safari().build()),
        ("Edge/146", Client::builder().edge().build()),
    ] {
        let c = client.expect(&format!("{name} should build"));
        assert_eq!(c.profile_name(), name);
    }
}

#[test]
fn invalid_default_header_name_returns_error() {
    let result = Client::builder()
        .chrome()
        .default_header("invalid\nheader", "value")
        .build();

    assert!(result.is_err(), "invalid header name should fail build");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("invalid header name"),
        "error should mention invalid header: {err}"
    );
}

#[test]
fn invalid_default_header_value_returns_error() {
    let result = Client::builder()
        .chrome()
        .default_header("x-test", "value\x00with\x01control\x02chars")
        .build();

    assert!(result.is_err(), "invalid header value should fail build");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("invalid header value"),
        "error should mention invalid value: {err}"
    );
}

#[test]
fn valid_default_headers_accepted() {
    let result = Client::builder()
        .chrome()
        .default_header("x-custom", "my-value")
        .default_header("authorization", "Bearer token123")
        .build();

    assert!(result.is_ok(), "valid headers should build");
}

#[test]
fn invalid_proxy_url_returns_error() {
    let result = Client::builder().chrome().proxy("not a url");

    assert!(result.is_err(), "invalid proxy should fail");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("invalid proxy"),
        "error should mention proxy: {err}"
    );
}

#[test]
fn valid_proxy_url_accepted() {
    let result = Client::builder()
        .chrome()
        .proxy("http://user:pass@proxy.example.com:8080");

    assert!(result.is_ok(), "valid proxy URL should be accepted");
}

#[test]
fn cookie_store_can_be_disabled() {
    let result = Client::builder().chrome().cookie_store(false).build();
    assert!(result.is_ok());
}

#[test]
fn custom_timeout_accepted() {
    let result = Client::builder()
        .chrome()
        .timeout(std::time::Duration::from_millis(500))
        .build();
    assert!(result.is_ok());
}

#[test]
fn bandwidth_tracker_is_shared() {
    let stats = webclaw_http::BandwidthStats::new();
    let client = Client::builder()
        .chrome()
        .bandwidth_tracker(stats.clone())
        .build()
        .expect("build");

    // Same tracker instance
    assert_eq!(client.bandwidth().request_count(), 0);
    assert_eq!(stats.request_count(), 0);
}

#[test]
fn header_order_override() {
    let custom = webclaw_http::HeaderOrder::custom(&["cookie", "user-agent"]);
    let client = Client::builder()
        .chrome()
        .header_order(custom)
        .build()
        .expect("build");

    // Should use custom order, not Chrome's default
    let mut headers = vec![
        ("user-agent".to_string(), "bot".to_string()),
        ("cookie".to_string(), "session=abc".to_string()),
    ];
    client.header_order().apply(&mut headers);
    assert_eq!(headers[0].0, "cookie");
    assert_eq!(headers[1].0, "user-agent");
}

// --- Error type ---

#[test]
fn error_display_formats() {
    let err = Error::from(url::Url::parse("://bad").unwrap_err());
    assert!(err.to_string().contains("invalid URL"));

    let err: Error = "test".parse::<url::Url>().unwrap_err().into();
    assert!(err.to_string().contains("invalid URL"));
}

#[test]
fn error_source_chain() {
    use std::error::Error as StdError;

    let parse_err = url::Url::parse("://bad").unwrap_err();
    let err = Error::from(parse_err);
    assert!(err.source().is_some(), "InvalidUrl should have source");
}
