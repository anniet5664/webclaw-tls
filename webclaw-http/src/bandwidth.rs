//! Bandwidth tracking for HTTP requests.
//!
//! Provides atomic byte counters that are shared across cloned clients.
//! All operations are lock-free.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Thread-safe bandwidth tracker shared across all clones of a client.
///
/// Uses atomic operations — no locks, safe to read from any thread while
/// requests are in flight.
#[derive(Debug, Clone)]
pub struct BandwidthStats {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    request_count: AtomicU64,
}

/// Bandwidth info for a single request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestBandwidth {
    /// Approximate bytes sent (headers + body). Excludes TLS overhead.
    pub sent: u64,
    /// Response body length in bytes (after decompression).
    pub received: u64,
}

impl BandwidthStats {
    /// Create a new tracker with all counters at zero.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                bytes_sent: AtomicU64::new(0),
                bytes_received: AtomicU64::new(0),
                request_count: AtomicU64::new(0),
            }),
        }
    }

    /// Record a completed request's bandwidth.
    pub(crate) fn record(&self, bw: RequestBandwidth) {
        self.inner.bytes_sent.fetch_add(bw.sent, Ordering::Relaxed);
        self.inner
            .bytes_received
            .fetch_add(bw.received, Ordering::Relaxed);
        self.inner.request_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Total bytes sent across all requests.
    #[must_use]
    pub fn total_sent(&self) -> u64 {
        self.inner.bytes_sent.load(Ordering::Relaxed)
    }

    /// Total bytes received across all requests.
    #[must_use]
    pub fn total_received(&self) -> u64 {
        self.inner.bytes_received.load(Ordering::Relaxed)
    }

    /// Total requests completed (success or failure).
    #[must_use]
    pub fn request_count(&self) -> u64 {
        self.inner.request_count.load(Ordering::Relaxed)
    }

    /// Reset all counters to zero.
    ///
    /// Note: not atomic across counters. A concurrent `record()` between
    /// individual resets may cause a brief inconsistency. This is acceptable
    /// for a stats-only API — use `snapshot()` if you need a consistent read.
    pub fn reset(&self) {
        self.inner.bytes_sent.store(0, Ordering::Relaxed);
        self.inner.bytes_received.store(0, Ordering::Relaxed);
        self.inner.request_count.store(0, Ordering::Relaxed);
    }

    /// Snapshot of all counters at approximately the same point in time.
    ///
    /// Not globally consistent (no memory fence across counters) but close
    /// enough for monitoring/logging. For exact consistency, stop sending
    /// requests before reading.
    #[must_use]
    pub fn snapshot(&self) -> BandwidthSnapshot {
        BandwidthSnapshot {
            sent: self.total_sent(),
            received: self.total_received(),
            requests: self.request_count(),
        }
    }
}

impl Default for BandwidthStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Point-in-time snapshot of bandwidth counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BandwidthSnapshot {
    pub sent: u64,
    pub received: u64,
    pub requests: u64,
}

impl std::fmt::Display for BandwidthSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} requests, {} sent, {} received",
            self.requests,
            format_bytes(self.sent),
            format_bytes(self.received),
        )
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

/// Estimate HTTP request size from its components.
///
/// Approximation — does not account for HTTP/2 HPACK compression or TLS record framing.
pub(crate) fn estimate_request_size(
    method: &str,
    url: &url::Url,
    headers: &[(String, String)],
    body_len: usize,
) -> u64 {
    let mut size: usize = 0;

    // HTTP/2 pseudo-headers: :method, :scheme, :authority, :path
    size += method.len();
    size += url.scheme().len();
    size += url.host_str().map_or(0, str::len);
    size += url.path().len();
    size += url.query().map_or(0, |q| q.len() + 1); // +1 for '?'
    size += 32; // frame overhead estimate

    // Regular headers
    for (name, value) in headers {
        size += name.len() + value.len() + 4; // ": " + "\r\n"
    }

    size += body_len;

    size as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_at_zero() {
        let stats = BandwidthStats::new();
        assert_eq!(stats.total_sent(), 0);
        assert_eq!(stats.total_received(), 0);
        assert_eq!(stats.request_count(), 0);
    }

    #[test]
    fn record_accumulates() {
        let stats = BandwidthStats::new();
        stats.record(RequestBandwidth {
            sent: 100,
            received: 500,
        });
        stats.record(RequestBandwidth {
            sent: 200,
            received: 1000,
        });
        assert_eq!(stats.total_sent(), 300);
        assert_eq!(stats.total_received(), 1500);
        assert_eq!(stats.request_count(), 2);
    }

    #[test]
    fn reset_clears_all() {
        let stats = BandwidthStats::new();
        stats.record(RequestBandwidth {
            sent: 100,
            received: 500,
        });
        stats.reset();
        assert_eq!(stats.total_sent(), 0);
        assert_eq!(stats.total_received(), 0);
        assert_eq!(stats.request_count(), 0);
    }

    #[test]
    fn clone_shares_state() {
        let a = BandwidthStats::new();
        let b = a.clone();
        a.record(RequestBandwidth {
            sent: 100,
            received: 200,
        });
        assert_eq!(b.total_sent(), 100);
    }

    #[test]
    fn snapshot_display() {
        let stats = BandwidthStats::new();
        stats.record(RequestBandwidth {
            sent: 1500,
            received: 2_500_000,
        });
        let snap = stats.snapshot();
        let display = format!("{snap}");
        assert!(display.contains("1.5 KB"));
        assert!(display.contains("2.4 MB"));
        assert!(display.contains("1 requests"));
    }
}
