//! HTTP client with browser fingerprinting.
//!
//! Configures reqwest with browser-specific TLS, HTTP/2, and header settings.
//! No primp dependency — uses our rustls + h2 forks via [patch.crates-io].

use std::collections::HashMap;
use std::error::Error as StdError;
use std::time::{Duration, Instant};

use crate::bandwidth::{BandwidthStats, RequestBandwidth};
use crate::error::Error;
use crate::header_order::HeaderOrder;
use crate::profiles::{self, BrowserProfile};

/// HTTP client with browser TLS fingerprinting.
///
/// Cheap to clone — clones share connection pool, cookie jar, bandwidth tracker.
#[derive(Clone)]
pub struct Client {
    inner: reqwest::Client,
    header_order: HeaderOrder,
    bandwidth: BandwidthStats,
    profile_name: &'static str,
}

/// Builder for [`Client`].
pub struct ClientBuilder {
    profile: Option<BrowserProfile>,
    timeout: Duration,
    proxy: Option<String>,
    header_order_override: Option<HeaderOrder>,
    bandwidth: BandwidthStats,
    extra_headers: Vec<(String, String)>,
    cookie_store: bool,
}

/// HTTP response.
pub struct Response {
    status: u16,
    url: String,
    headers: HashMap<String, String>,
    body: bytes::Bytes,
    elapsed: Duration,
    bw: RequestBandwidth,
}

impl Client {
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder {
            profile: None,
            timeout: Duration::from_secs(30),
            proxy: None,
            header_order_override: None,
            bandwidth: BandwidthStats::new(),
            extra_headers: Vec::new(),
            cookie_store: true,
        }
    }

    pub async fn get(&self, url: &str) -> Result<Response, Error> {
        self.execute(reqwest::Method::GET, url, None).await
    }

    pub async fn post(&self, url: &str, body: &[u8]) -> Result<Response, Error> {
        self.execute(reqwest::Method::POST, url, Some(body)).await
    }

    pub async fn put(&self, url: &str, body: &[u8]) -> Result<Response, Error> {
        self.execute(reqwest::Method::PUT, url, Some(body)).await
    }

    pub async fn delete(&self, url: &str) -> Result<Response, Error> {
        self.execute(reqwest::Method::DELETE, url, None).await
    }

    pub async fn patch(&self, url: &str, body: &[u8]) -> Result<Response, Error> {
        self.execute(reqwest::Method::PATCH, url, Some(body)).await
    }

    pub async fn head(&self, url: &str) -> Result<Response, Error> {
        self.execute(reqwest::Method::HEAD, url, None).await
    }

    async fn execute(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<&[u8]>,
    ) -> Result<Response, Error> {
        let start = Instant::now();

        let mut req = self.inner.request(method.clone(), url);
        if let Some(b) = body {
            req = req.body(bytes::Bytes::copy_from_slice(b));
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();
        let final_url = resp.url().as_str().to_string();

        // Pre-allocate header map based on header count
        let header_count = resp.headers().len();
        let mut headers = HashMap::with_capacity(header_count);
        for (k, v) in resp.headers().iter() {
            headers.insert(
                k.as_str().to_string(),
                v.to_str().unwrap_or("").to_string(),
            );
        }

        // Single body read — Bytes is reference-counted, no extra copy
        let body_bytes = resp
            .bytes()
            .await
            .map_err(|e| Error::BodyDecode(e.to_string()))?;

        let elapsed = start.elapsed();
        let received = body_bytes.len() as u64;
        let sent = (method.as_str().len() + url.len() + 200) as u64; // rough estimate without allocation

        let bw = RequestBandwidth { sent, received };
        self.bandwidth.record(bw);

        Ok(Response {
            status,
            url: final_url,
            headers,
            body: body_bytes,
            elapsed,
            bw,
        })
    }

    #[must_use]
    pub fn bandwidth(&self) -> &BandwidthStats {
        &self.bandwidth
    }

    #[must_use]
    pub fn header_order(&self) -> &HeaderOrder {
        &self.header_order
    }

    #[must_use]
    pub fn profile_name(&self) -> &str {
        self.profile_name
    }
}

impl ClientBuilder {
    #[must_use]
    pub fn chrome(mut self) -> Self {
        self.profile = Some(profiles::chrome());
        self
    }

    #[must_use]
    pub fn chrome_macos(mut self) -> Self {
        self.profile = Some(profiles::chrome_macos());
        self
    }

    #[must_use]
    pub fn firefox(mut self) -> Self {
        self.profile = Some(profiles::firefox());
        self
    }

    #[must_use]
    pub fn safari(mut self) -> Self {
        self.profile = Some(profiles::safari());
        self
    }

    #[must_use]
    pub fn edge(mut self) -> Self {
        self.profile = Some(profiles::edge());
        self
    }

    #[must_use]
    pub fn profile(mut self, profile: BrowserProfile) -> Self {
        self.profile = Some(profile);
        self
    }

    #[must_use]
    pub fn header_order(mut self, order: HeaderOrder) -> Self {
        self.header_order_override = Some(order);
        self
    }

    #[must_use]
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    pub fn proxy(mut self, url: &str) -> Result<Self, Error> {
        self.proxy = Some(url.to_string());
        Ok(self)
    }

    #[must_use]
    pub fn default_header(mut self, name: &str, value: &str) -> Self {
        self.extra_headers.push((name.to_string(), value.to_string()));
        self
    }

    #[must_use]
    pub fn cookie_store(mut self, enable: bool) -> Self {
        self.cookie_store = enable;
        self
    }

    #[must_use]
    pub fn bandwidth_tracker(mut self, stats: BandwidthStats) -> Self {
        self.bandwidth = stats;
        self
    }

    pub fn build(self) -> Result<Client, Error> {
        let profile = self.profile.unwrap_or_else(profiles::chrome);
        let header_order = self.header_order_override.unwrap_or(profile.header_order.clone());

        let tls_config: Option<reqwest::rustls::ClientConfig> =
            Some(crate::tls::build_tls_config(&profile)?);

        let h2 = &profile.h2_settings;

        use h2::frame::{PseudoId, PseudoOrder, SettingId, SettingsOrder};

        let settings_order = SettingsOrder::builder()
            .push(SettingId::HeaderTableSize)
            .push(SettingId::EnablePush)
            .push(SettingId::InitialWindowSize)
            .push(SettingId::MaxHeaderListSize)
            .build();

        let is_chrome = profile.name.starts_with("Chrome")
            || profile.name.starts_with("Edge")
            || profile.name.starts_with("Opera");

        let pseudo_order = if is_chrome {
            PseudoOrder::builder()
                .push(PseudoId::Method)
                .push(PseudoId::Authority)
                .push(PseudoId::Scheme)
                .push(PseudoId::Path)
                .build()
        } else {
            PseudoOrder::builder()
                .push(PseudoId::Method)
                .push(PseudoId::Path)
                .push(PseudoId::Authority)
                .push(PseudoId::Scheme)
                .build()
        };

        let mut builder = reqwest::Client::builder()
            .rustls_config(tls_config.unwrap())
            .user_agent(profile.user_agent)
            .timeout(self.timeout)
            .cookie_store(self.cookie_store)
            .http2_header_table_size(h2.header_table_size)
            .http2_enable_push(h2.enable_push)
            .http2_initial_stream_window_size(h2.initial_window_size)
            .http2_initial_connection_window_size(profile.h2_connection_window)
            .http2_settings_order(settings_order)
            .http2_headers_pseudo_order(pseudo_order);

        if h2.max_header_list_size > 0 {
            builder = builder.http2_max_header_list_size(h2.max_header_list_size);
        }
        if let Some(max) = h2.max_concurrent_streams {
            builder = builder.http2_max_concurrent_streams(max);
        }
        if let Some(max) = h2.max_frame_size {
            builder = builder.http2_max_frame_size(max);
        }

        let mut header_map = reqwest::header::HeaderMap::with_capacity(profile.default_headers.len());
        for (name, value) in profile.default_headers {
            if let (Ok(n), Ok(v)) = (
                reqwest::header::HeaderName::from_bytes(name.as_bytes()),
                reqwest::header::HeaderValue::from_str(value),
            ) {
                header_map.insert(n, v);
            }
        }
        for (name, value) in &self.extra_headers {
            if let (Ok(n), Ok(v)) = (
                reqwest::header::HeaderName::from_bytes(name.as_bytes()),
                reqwest::header::HeaderValue::from_str(value),
            ) {
                header_map.insert(n, v);
            }
        }
        builder = builder.default_headers(header_map);

        if let Some(proxy_url) = &self.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| Error::Build(format!("invalid proxy: {e}")))?;
            builder = builder.proxy(proxy);
        }

        let inner = builder.build().map_err(|e| {
            let source_msg = StdError::source(&e).map_or(String::new(), |s| format!(": {s}"));
            Error::Build(format!("{e}{source_msg}"))
        })?;

        Ok(Client {
            inner,
            header_order,
            bandwidth: self.bandwidth,
            profile_name: profile.name,
        })
    }
}

// --- Response ---

impl Response {
    #[must_use]
    pub fn status(&self) -> u16 {
        self.status
    }

    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    #[must_use]
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Raw body bytes. Zero-copy reference to the response buffer.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Body as UTF-8 text. Returns a Cow to avoid allocation when possible.
    #[must_use]
    pub fn text(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }

    /// Consume the response and return body as String.
    #[must_use]
    pub fn into_text(self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    #[must_use]
    pub fn bandwidth(&self) -> RequestBandwidth {
        self.bw
    }

    #[must_use]
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_ascii_lowercase()).map(String::as_str)
    }

    #[must_use]
    pub fn content_type(&self) -> Option<&str> {
        self.header("content-type")
    }
}
