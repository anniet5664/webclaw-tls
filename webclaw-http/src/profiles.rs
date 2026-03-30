//! Browser fingerprint profiles.
//!
//! Each profile defines the complete set of parameters needed to impersonate
//! a real browser: TLS config (via rustls features), HTTP/2 settings,
//! default headers, header order, and User-Agent.
//!
//! Profiles are captured from real browsers using tls.peet.ws/api/all.

use crate::header_order::HeaderOrder;

/// HTTP/2 SETTINGS identifier for frame ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H2Setting {
    HeaderTableSize,
    EnablePush,
    InitialWindowSize,
    MaxConcurrentStreams,
    MaxHeaderListSize,
    MaxFrameSize,
}

/// HTTP/2 pseudo-header for ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PseudoHeader {
    Method,
    Authority,
    Scheme,
    Path,
}

/// Complete browser fingerprint profile.
#[derive(Debug, Clone)]
pub struct BrowserProfile {
    /// Display name for logging.
    pub name: &'static str,
    /// User-Agent string.
    pub user_agent: &'static str,
    /// HTTP/2 SETTINGS values.
    pub h2_settings: H2Settings,
    /// HTTP/2 initial connection window size.
    pub h2_connection_window: u32,
    /// Default HTTP headers (in wire order).
    pub default_headers: &'static [(&'static str, &'static str)],
    /// Header ordering for this browser.
    pub header_order: HeaderOrder,
    /// HTTP/2 SETTINGS frame wire order.
    pub settings_order: &'static [H2Setting],
    /// HTTP/2 pseudo-header wire order.
    pub pseudo_order: &'static [PseudoHeader],
}

impl BrowserProfile {
    /// Whether this is a Chromium-based browser (Chrome, Edge, Opera).
    #[must_use]
    pub fn is_chromium(&self) -> bool {
        matches!(
            self.name.split('/').next(),
            Some("Chrome" | "Edge" | "Opera")
        )
    }

    /// Whether this is a Firefox browser.
    #[must_use]
    pub fn is_firefox(&self) -> bool {
        self.name.starts_with("Firefox")
    }
}

/// HTTP/2 SETTINGS frame configuration.
#[derive(Debug, Clone, Copy)]
pub struct H2Settings {
    pub header_table_size: u32,
    pub enable_push: bool,
    pub initial_window_size: u32,
    pub max_header_list_size: u32,
    pub max_concurrent_streams: Option<u32>,
    pub max_frame_size: Option<u32>,
}

/// Chrome 146 on Windows — captured from real Chrome via tls.peet.ws.
pub fn chrome() -> BrowserProfile {
    BrowserProfile {
        name: "Chrome/146",
        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36",
        h2_settings: H2Settings {
            header_table_size: 65536,
            enable_push: false,
            initial_window_size: 6291456,
            max_header_list_size: 262144,
            max_concurrent_streams: None,
            max_frame_size: None,
        },
        // Connection window = default(65535) + WINDOW_UPDATE.
        // Chrome sends WINDOW_UPDATE of 15663105, so total = 65535 + 15663105 = 15728640.
        // reqwest sets: WINDOW_UPDATE = connection_window - 65535.
        h2_connection_window: 15728640,
        default_headers: &[
            ("sec-ch-ua", "\"Chromium\";v=\"146\", \"Not-A.Brand\";v=\"24\", \"Google Chrome\";v=\"146\""),
            ("sec-ch-ua-mobile", "?0"),
            ("sec-ch-ua-platform", "\"Windows\""),
            ("upgrade-insecure-requests", "1"),
            // user-agent is set separately via reqwest builder
            ("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"),
            ("sec-fetch-site", "none"),
            ("sec-fetch-mode", "navigate"),
            ("sec-fetch-user", "?1"),
            ("sec-fetch-dest", "document"),
            ("accept-encoding", "gzip, deflate, br, zstd"),
            ("accept-language", "en-US,en;q=0.9"),
            ("priority", "u=0, i"),
        ],
        header_order: HeaderOrder::chrome(),
        settings_order: &[
            H2Setting::HeaderTableSize,
            H2Setting::EnablePush,
            H2Setting::InitialWindowSize,
            H2Setting::MaxHeaderListSize,
        ],
        pseudo_order: &[
            PseudoHeader::Method,
            PseudoHeader::Authority,
            PseudoHeader::Scheme,
            PseudoHeader::Path,
        ],
    }
}

/// Chrome 146 on macOS.
pub fn chrome_macos() -> BrowserProfile {
    let mut p = chrome();
    p.user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36";
    p.default_headers = &[
        ("sec-ch-ua", "\"Chromium\";v=\"146\", \"Not-A.Brand\";v=\"24\", \"Google Chrome\";v=\"146\""),
        ("sec-ch-ua-mobile", "?0"),
        ("sec-ch-ua-platform", "\"macOS\""),
        ("upgrade-insecure-requests", "1"),
        ("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"),
        ("sec-fetch-site", "none"),
        ("sec-fetch-mode", "navigate"),
        ("sec-fetch-user", "?1"),
        ("sec-fetch-dest", "document"),
        ("accept-encoding", "gzip, deflate, br, zstd"),
        ("accept-language", "en-US,en;q=0.9"),
        ("priority", "u=0, i"),
    ];
    p
}

/// Firefox 146 on Windows — captured from real Firefox.
pub fn firefox() -> BrowserProfile {
    BrowserProfile {
        name: "Firefox/146",
        user_agent:
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:146.0) Gecko/20100101 Firefox/146.0",
        h2_settings: H2Settings {
            header_table_size: 65536,
            enable_push: true,
            initial_window_size: 131072,
            max_header_list_size: 65536,
            max_concurrent_streams: None,
            max_frame_size: None,
        },
        h2_connection_window: 12517377,
        default_headers: &[
            // user-agent set separately
            (
                "accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
            ("accept-language", "en-US,en;q=0.5"),
            ("accept-encoding", "gzip, deflate, br, zstd"),
            ("upgrade-insecure-requests", "1"),
            ("sec-fetch-dest", "document"),
            ("sec-fetch-mode", "navigate"),
            ("sec-fetch-site", "none"),
            ("sec-fetch-user", "?1"),
            ("priority", "u=0, i"),
            ("te", "trailers"),
        ],
        header_order: HeaderOrder::firefox(),
        settings_order: &[
            H2Setting::HeaderTableSize,
            H2Setting::EnablePush,
            H2Setting::InitialWindowSize,
            H2Setting::MaxHeaderListSize,
        ],
        pseudo_order: &[
            PseudoHeader::Method,
            PseudoHeader::Path,
            PseudoHeader::Authority,
            PseudoHeader::Scheme,
        ],
    }
}

/// Safari 18 on macOS — captured from real Safari via tls.peet.ws.
pub fn safari() -> BrowserProfile {
    BrowserProfile {
        name: "Safari/18",
        user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.5 Safari/605.1.15",
        h2_settings: H2Settings {
            header_table_size: 4096,
            enable_push: true,
            initial_window_size: 2097152,
            max_header_list_size: 0, // Safari doesn't set this
            max_concurrent_streams: Some(100),
            max_frame_size: Some(16384),
        },
        h2_connection_window: 10485760,
        default_headers: &[
            ("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
            ("sec-fetch-site", "none"),
            ("sec-fetch-dest", "document"),
            ("accept-language", "en-GB,en;q=0.9"),
            ("sec-fetch-mode", "navigate"),
            // user-agent set separately
            ("accept-encoding", "gzip, deflate, br"),
            ("priority", "u=0, i"),
        ],
        header_order: HeaderOrder::safari(),
        settings_order: &[
            H2Setting::HeaderTableSize,
            H2Setting::EnablePush,
            H2Setting::InitialWindowSize,
            H2Setting::MaxConcurrentStreams,
            H2Setting::MaxFrameSize,
        ],
        pseudo_order: &[
            PseudoHeader::Method,
            PseudoHeader::Path,
            PseudoHeader::Authority,
            PseudoHeader::Scheme,
        ],
    }
}

/// Edge 146 on Windows (Chromium-based, same TLS fingerprint as Chrome).
pub fn edge() -> BrowserProfile {
    let mut p = chrome();
    p.name = "Edge/146";
    p.user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/146.0.0.0 Safari/537.36 Edg/146.0.0.0";
    p.default_headers = &[
        ("sec-ch-ua", "\"Chromium\";v=\"146\", \"Not-A.Brand\";v=\"24\", \"Microsoft Edge\";v=\"146\""),
        ("sec-ch-ua-mobile", "?0"),
        ("sec-ch-ua-platform", "\"Windows\""),
        ("upgrade-insecure-requests", "1"),
        ("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"),
        ("sec-fetch-site", "none"),
        ("sec-fetch-mode", "navigate"),
        ("sec-fetch-user", "?1"),
        ("sec-fetch-dest", "document"),
        ("accept-encoding", "gzip, deflate, br, zstd"),
        ("accept-language", "en-US,en;q=0.9"),
        ("priority", "u=0, i"),
    ];
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chrome_profile_has_correct_h2_settings() {
        let p = chrome();
        assert_eq!(p.h2_settings.header_table_size, 65536);
        assert!(!p.h2_settings.enable_push);
        assert_eq!(p.h2_settings.initial_window_size, 6291456);
        // 65535 (default) + 15663105 (WINDOW_UPDATE) = 15728640
        assert_eq!(p.h2_connection_window, 15728640);
    }

    #[test]
    fn firefox_has_different_h2_from_chrome() {
        let c = chrome();
        let f = firefox();
        assert_ne!(
            c.h2_settings.initial_window_size,
            f.h2_settings.initial_window_size
        );
        assert_ne!(c.h2_connection_window, f.h2_connection_window);
    }

    #[test]
    fn all_profiles_have_user_agent() {
        for p in [chrome(), firefox(), safari(), edge()] {
            assert!(!p.user_agent.is_empty());
            assert!(p.user_agent.contains("Mozilla"));
        }
    }

    #[test]
    fn all_profiles_have_accept_header() {
        for p in [chrome(), firefox(), safari(), edge()] {
            assert!(p.default_headers.iter().any(|(k, _)| *k == "accept"));
        }
    }

    #[test]
    fn is_chromium_detects_correctly() {
        assert!(chrome().is_chromium());
        assert!(chrome_macos().is_chromium());
        assert!(edge().is_chromium());
        assert!(!firefox().is_chromium());
        assert!(!safari().is_chromium());
    }

    #[test]
    fn is_firefox_detects_correctly() {
        assert!(firefox().is_firefox());
        assert!(!chrome().is_firefox());
        assert!(!safari().is_firefox());
    }

    #[test]
    fn chrome_and_firefox_have_different_pseudo_order() {
        assert_ne!(chrome().pseudo_order, firefox().pseudo_order);
    }

    #[test]
    fn safari_settings_include_concurrent_streams() {
        let s = safari();
        assert!(s.settings_order.contains(&H2Setting::MaxConcurrentStreams));
        assert!(!chrome()
            .settings_order
            .contains(&H2Setting::MaxConcurrentStreams));
    }
}
