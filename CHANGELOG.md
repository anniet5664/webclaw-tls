# Changelog

All notable changes to webclaw-tls will be documented in this file.

## [0.1.2] - 2026-03-30

### Fixed
- **HTTP header wire order** — `accept` and `user-agent` were at positions 4-5 instead of 8-9. Caused by reqwest pre-inserting `accept: */*` at position 0 in `ClientBuilder::new()`, pinning it ahead of `sec-ch-ua` headers. Fixed by replacing the header map instead of merging, and moving `user-agent` into each profile's `default_headers` at the correct wire position.
- **HTTP/2 PRIORITY flag missing in HEADERS frame** — Real Chrome sends HEADERS with PRIORITY flag (weight=256, dep=0, exclusive). Added `h2_headers_priority` field to `BrowserProfile`, wired through reqwest → hyper → h2. Chrome/Edge/Safari profiles now include PRIORITY; Firefox correctly omits it (deprecated).

### Changed
- `reqwest::ClientBuilder::default_headers()` now replaces the entire header map instead of merging into the pre-existing one, preserving caller's insertion order for HTTP fingerprinting.
- Removed `.user_agent()` call from client builder — UA is now set via `default_headers` at the correct position per browser profile.

## [0.1.1] - 2026-03-30

### Added
- `H2Setting` / `PseudoHeader` enums — per-browser HTTP/2 SETTINGS and pseudo-header wire order
- `BrowserProfile::is_chromium()` / `is_firefox()` methods
- `Response::into_bytes()` for zero-copy binary body consumption
- `Debug` derive on `Client`, `ClientBuilder`, `Response`
- `Send + Sync` compile-time assertions for all public types
- `pub use http::header::HeaderMap` re-export
- Hermetic unit tests (13 tests, no network required)
- Safari and Edge fingerprint integration tests
- GitHub Actions CI (fmt, clippy, unit tests, integration, downstream sync)
- Automated dependency sync: tls push propagates to core and server repos

### Changed
- `Response::headers()` returns `&http::header::HeaderMap` instead of `&HashMap<String, String>` — avoids per-response allocation, preserves multi-value headers
- `Response::header()` uses native `HeaderMap::get()` — case-insensitive by design, no manual lowercasing
- `ClientBuilder::proxy()` validates URL eagerly instead of deferring to `build()`
- `ClientBuilder::build()` returns errors on invalid header names/values instead of silently dropping them
- Bandwidth sent estimate now includes request body length
- SETTINGS order and pseudo-header order driven by `BrowserProfile` data instead of hardcoded in builder
- Browser detection uses `profile.is_chromium()` / `profile.is_firefox()` instead of string matching

### Removed
- Dead `estimate_request_size()` function (was defined but never called)
- Dead `CHROME_PSEUDO_ORDER` / `FIREFOX_PSEUDO_ORDER` constants
- Unnecessary `Option` wrapping of TLS config in builder

## [0.1.0] - 2026-03-29

### Added
- Initial release
- Browser-grade TLS fingerprinting via patched rustls (JA4 match)
- HTTP/2 fingerprinting via patched h2 (Akamai hash match)
- Chrome 146, Firefox 146, Safari 18, Edge 146 profiles
- Header ordering per browser
- Bandwidth tracking (atomic, lock-free)
- Cookie jar (thread-safe)
- All HTTP methods (GET, POST, PUT, DELETE, PATCH, HEAD)
- Proxy support, custom timeouts
- 89% bypass rate on 9 protected sites
