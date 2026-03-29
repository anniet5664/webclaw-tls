//! # webclaw-http
//!
//! HTTP client with browser TLS + HTTP/2 fingerprinting.
//!
//! Built on:
//! - **webclaw-rustls**: our patched rustls with correct JA4 fingerprint
//!   (extension order, PSK, GREASE — matches real Chrome/Firefox/Safari)
//! - **webclaw-h2**: our patched h2 with HTTP/2 SETTINGS ordering and
//!   pseudo-header order (matches real browser Akamai fingerprints)
//! - **reqwest**: HTTP client (patched reqwest that exposes h2 config)
//!
//! No dependency on primp. We own the TLS and HTTP/2 fingerprinting stack.
//!
//! ## Features
//!
//! - **TLS fingerprinting**: JA4 matches real Chrome 146, Firefox 135+, Safari 18
//! - **HTTP/2 fingerprinting**: Akamai hash matches real browsers
//! - **Header ordering**: per-browser HTTP header order
//! - **Bandwidth tracking**: atomic byte counters across all requests
//! - **Cookie jar**: thread-safe, inspectable
//! - **All HTTP methods**: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS

pub mod bandwidth;
pub mod error;
pub mod header_order;
pub mod profiles;

mod client;
mod tls;

pub use bandwidth::BandwidthStats;
pub use client::{Client, ClientBuilder, Response};
pub use error::Error;
pub use header_order::HeaderOrder;
