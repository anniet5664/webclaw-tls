//! Explicit HTTP header ordering.
//!
//! Browsers send HTTP headers in a specific, deterministic order that advanced
//! fingerprint detectors (Akamai, Cloudflare) can check. This module provides
//! per-browser header orders captured from real browsers via tls.peet.ws.
//!
//! # Custom ordering
//!
//! ```
//! use webclaw_http::HeaderOrder;
//!
//! let order = HeaderOrder::custom(&["user-agent", "accept", "cookie"]);
//! ```

use std::borrow::Cow;

/// HTTP header ordering configuration.
///
/// Defines the order in which HTTP headers are sent on the wire.
/// Headers present in the order list are sent first (in the specified order),
/// followed by any remaining headers not in the list.
#[derive(Debug, Clone)]
pub struct HeaderOrder {
    /// Lowercased header names in the desired wire order.
    names: Vec<Cow<'static, str>>,
}

impl HeaderOrder {
    /// Chrome 146 header order (captured from real Chrome via tls.peet.ws/api/all).
    ///
    /// Pseudo-headers (:method, :authority, :scheme, :path) are handled at the
    /// HTTP/2 layer, not here. This covers regular headers only.
    #[must_use]
    pub fn chrome() -> Self {
        Self::from_static(&[
            "sec-ch-ua",
            "sec-ch-ua-mobile",
            "sec-ch-ua-platform",
            "upgrade-insecure-requests",
            "user-agent",
            "accept",
            "sec-fetch-site",
            "sec-fetch-mode",
            "sec-fetch-user",
            "sec-fetch-dest",
            "accept-encoding",
            "accept-language",
            "priority",
            "cookie",
        ])
    }

    /// Firefox 135+ header order (captured from real Firefox).
    #[must_use]
    pub fn firefox() -> Self {
        Self::from_static(&[
            "user-agent",
            "accept",
            "accept-language",
            "accept-encoding",
            "cookie",
            "upgrade-insecure-requests",
            "sec-fetch-dest",
            "sec-fetch-mode",
            "sec-fetch-site",
            "sec-fetch-user",
            "priority",
            "te",
        ])
    }

    /// Safari 18 header order (captured from real Safari on macOS).
    #[must_use]
    pub fn safari() -> Self {
        Self::from_static(&[
            "accept",
            "sec-fetch-site",
            "cookie",
            "sec-fetch-dest",
            "accept-language",
            "sec-fetch-mode",
            "user-agent",
            "accept-encoding",
            "priority",
        ])
    }

    /// Custom header order from a list of header names.
    ///
    /// Names are lowercased automatically. Headers not in this list will be
    /// appended after the ordered headers in their original insertion order.
    #[must_use]
    pub fn custom(names: &[&str]) -> Self {
        Self {
            names: names
                .iter()
                .map(|n| Cow::Owned(n.to_ascii_lowercase()))
                .collect(),
        }
    }

    /// Sort a header list according to this ordering.
    ///
    /// Uses a stable sort — headers with the same priority retain their
    /// relative insertion order.
    pub fn apply(&self, headers: &mut [(String, String)]) {
        headers.sort_by(|(a, _), (b, _)| {
            let pos_a = self.position(a);
            let pos_b = self.position(b);
            pos_a.cmp(&pos_b)
        });
    }

    /// Position of a header name in the order. Unordered headers sort last.
    fn position(&self, name: &str) -> usize {
        let lower = name.to_ascii_lowercase();
        self.names
            .iter()
            .position(|n| n.as_ref() == lower)
            .unwrap_or(usize::MAX)
    }

    /// Build from a static slice (zero allocation for built-in profiles).
    fn from_static(names: &[&'static str]) -> Self {
        Self {
            names: names.iter().map(|&n| Cow::Borrowed(n)).collect(),
        }
    }
}

impl Default for HeaderOrder {
    fn default() -> Self {
        Self::chrome()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chrome_order_sorts_correctly() {
        let order = HeaderOrder::chrome();
        let mut headers = vec![
            ("accept-encoding".into(), "gzip".into()),
            ("sec-ch-ua".into(), "chromium".into()),
            ("user-agent".into(), "Mozilla".into()),
            ("accept".into(), "*/*".into()),
        ];
        order.apply(&mut headers);

        let names: Vec<&str> = headers.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            names,
            &["sec-ch-ua", "user-agent", "accept", "accept-encoding"]
        );
    }

    #[test]
    fn unordered_headers_go_last() {
        let order = HeaderOrder::chrome();
        let mut headers = vec![
            ("x-custom".into(), "value".into()),
            ("user-agent".into(), "Mozilla".into()),
            ("x-other".into(), "value2".into()),
        ];
        order.apply(&mut headers);

        assert_eq!(headers[0].0, "user-agent");
        // x-custom and x-other should be after user-agent
        assert!(headers[1].0.starts_with("x-"));
        assert!(headers[2].0.starts_with("x-"));
    }

    #[test]
    fn custom_order() {
        let order = HeaderOrder::custom(&["Cookie", "User-Agent"]);
        let mut headers = vec![
            ("user-agent".into(), "bot".into()),
            ("accept".into(), "*/*".into()),
            ("cookie".into(), "session=abc".into()),
        ];
        order.apply(&mut headers);

        assert_eq!(headers[0].0, "cookie");
        assert_eq!(headers[1].0, "user-agent");
        assert_eq!(headers[2].0, "accept");
    }

    #[test]
    fn case_insensitive() {
        let order = HeaderOrder::chrome();
        let mut headers = vec![
            ("Accept-Encoding".into(), "gzip".into()),
            ("SEC-CH-UA".into(), "chromium".into()),
        ];
        order.apply(&mut headers);
        assert_eq!(headers[0].0, "SEC-CH-UA");
    }

    #[test]
    fn static_profiles_no_allocation() {
        let chrome = HeaderOrder::chrome();
        // Cow::Borrowed means no heap allocation for the strings
        assert!(chrome.names.iter().all(|n| matches!(n, Cow::Borrowed(_))));
    }
}
