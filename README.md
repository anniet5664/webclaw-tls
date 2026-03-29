<p align="center">
  <a href="https://webclaw.io">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/0xMassi/webclaw/main/.github/banner.png" />
      <img src="https://raw.githubusercontent.com/0xMassi/webclaw/main/.github/banner.png" alt="webclaw" width="700" />
    </picture>
  </a>
</p>

<h3 align="center">
  Browser-grade TLS + HTTP/2 fingerprinting for Rust.<br/>
  <sub>Perfect Chrome 146 JA4. Perfect Akamai hash. Zero unsafe.</sub>
</h3>

<p align="center">
  <a href="https://github.com/0xMassi/webclaw-tls/stargazers"><img src="https://img.shields.io/github/stars/0xMassi/webclaw-tls?style=for-the-badge&logo=github&logoColor=white&label=Stars&color=181717" alt="Stars" /></a>
  <a href="https://github.com/0xMassi/webclaw-tls/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-10B981?style=for-the-badge" alt="License" /></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-stable-B7410E?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" /></a>
</p>
<p align="center">
  <a href="https://discord.gg/KDfd48EpnW"><img src="https://img.shields.io/badge/Discord-Join-5865F2?style=for-the-badge&logo=discord&logoColor=white" alt="Discord" /></a>
  <a href="https://x.com/webclaw_io"><img src="https://img.shields.io/badge/Follow-@webclaw__io-000000?style=for-the-badge&logo=x&logoColor=white" alt="X / Twitter" /></a>
  <a href="https://webclaw.io"><img src="https://img.shields.io/badge/Website-webclaw.io-0A0A0A?style=for-the-badge&logo=safari&logoColor=white" alt="Website" /></a>
</p>

---

Your HTTP client says it's Chrome, but Cloudflare knows it's not. **JA4 fingerprints don't lie.**

`webclaw-tls` is a set of surgical patches to [rustls](https://github.com/rustls/rustls) and [h2](https://github.com/hyperium/h2) that make your Rust HTTP client indistinguishable from a real browser at the TLS and HTTP/2 protocol level. It's the only library in any language with a **perfect Chrome 146 JA4 + Akamai fingerprint match**.

Built for [webclaw](https://github.com/0xMassi/webclaw), the fastest web scraper for AI agents.

---

## Benchmark

### TLS Fingerprint Accuracy

| Library | Language | JA4 | Chrome 146 Match | Akamai Match |
|---|---|---|---|---|
| **webclaw-tls** | **Rust** | `t13d1517h2_8daaf6152771_b6f405a00624` | **PERFECT** | **PERFECT** |
| bogdanfinn/tls-client | Go | `t13d1517h2_8daaf6152771_dcad5a053991` | Close (wrong ext hash) | PERFECT |
| curl_cffi | Python/C | `t13d1516h2_8daaf6152771_d8a2da3f94cd` | No (missing PSK) | PERFECT |
| got-scraping | Node.js | `t13d1513h2_8daaf6152771_ff9cead5a15b` | No (4 exts missing) | No |
| primp | Rust | `t13d1516h2_8daaf6152771_d8a2da3f94cd` | No (wrong ext hash) | PERFECT |

### Site Bypass Rate (9 protected sites)

| Site | webclaw-tls | bogdanfinn | curl_cffi | got-scraping |
|---|---|---|---|---|
| Nike | OK | OK | OK | OK |
| Cloudflare | OK | BLOCKED | OK | OK |
| Zillow | OK | BLOCKED | OK | BLOCKED |
| Viagogo | OK | OK | OK | OK |
| Fansale | OK | ERROR | OK | OK |
| StockX | BLOCKED (JS) | BLOCKED | BLOCKED | BLOCKED |
| Indeed | OK | BLOCKED | OK | BLOCKED |
| Bloomberg | OK | BLOCKED | OK | BLOCKED |
| **PASS RATE** | **8/9 (89%)** | **3/9 (33%)** | **8/9 (89%)** | **5/9 (56%)** |

### Speed (fair comparison — only sites where both get real content)

| Metric | webclaw-tls | bogdanfinn |
|---|---|---|
| Cold (new TLS) | 413ms | 314ms |
| Warm (HTTP/2 reuse) | 163ms | 158ms |
| Fair average (real content) | **339ms** | 399ms |

Warm connections (real-world usage) are at parity. On fair comparisons we're **15% faster**.

---

## Quick Start

Add `webclaw-http` to your project:

```toml
# Cargo.toml
[dependencies]
webclaw-http = { git = "https://github.com/0xMassi/webclaw-tls" }
tokio = { version = "1", features = ["full"] }

# Required: patch crates-io to use our fingerprinting forks
[patch.crates-io]
rustls = { git = "https://github.com/0xMassi/webclaw-tls", package = "rustls" }
h2 = { git = "https://github.com/0xMassi/webclaw-tls", package = "h2" }
hyper = { git = "https://github.com/0xMassi/webclaw-tls", package = "hyper" }
hyper-util = { git = "https://github.com/0xMassi/webclaw-tls", package = "hyper-util" }
reqwest = { git = "https://github.com/0xMassi/webclaw-tls", package = "reqwest" }
```

```rust
use webclaw_http::Client;

#[tokio::main]
async fn main() {
    let client = Client::builder()
        .chrome()       // Perfect Chrome 146 fingerprint
        .build()
        .expect("build client");

    let resp = client.get("https://www.cloudflare.com").await.unwrap();
    println!("{} — {} bytes", resp.status(), resp.body().len());
}
```

That's it. Your requests now have a perfect Chrome 146 JA4 + HTTP/2 Akamai fingerprint.

---

## Browser Profiles

| Profile | Builder | JA4 Match | User-Agent |
|---|---|---|---|
| Chrome 146 (Win) | `.chrome()` | Perfect | Chrome/146.0.0.0 Windows |
| Chrome 146 (Mac) | `.chrome_macos()` | Perfect | Chrome/146.0.0.0 macOS |
| Firefox 135+ | `.firefox()` | Perfect | Firefox/135.0 |
| Safari 18 | `.safari()` | Perfect | Safari 18.3.1 macOS |
| Edge 146 | `.edge()` | Perfect | Edge/146.0.0.0 |

All profiles captured from real browsers via [tls.peet.ws](https://tls.peet.ws).

---

## Features

```rust
// Browser impersonation
let client = Client::builder().chrome().build()?;
let client = Client::builder().firefox().build()?;
let client = Client::builder().safari().build()?;

// All HTTP methods
let resp = client.get("https://example.com").await?;
let resp = client.post("https://example.com/api", b"body").await?;
let resp = client.put("https://example.com/api", b"data").await?;
let resp = client.delete("https://example.com/api").await?;
let resp = client.patch("https://example.com/api", b"patch").await?;
let resp = client.head("https://example.com").await?;

// Proxy support
let client = Client::builder()
    .chrome()
    .proxy("http://user:pass@proxy:8080")?
    .build()?;

// Custom timeout
let client = Client::builder()
    .chrome()
    .timeout(std::time::Duration::from_secs(10))
    .build()?;

// Bandwidth tracking (shared across clones)
use webclaw_http::BandwidthStats;
let stats = BandwidthStats::new();
let client = Client::builder()
    .chrome()
    .bandwidth_tracker(stats.clone())
    .build()?;
client.get("https://example.com").await?;
println!("{}", stats.snapshot()); // "1 requests, 0 B sent, 45.2 KB received"

// Cookie jar (enabled by default, thread-safe)
let client = Client::builder().chrome().build()?;
client.get("https://httpbin.org/cookies/set/foo/bar").await?;
let resp = client.get("https://httpbin.org/cookies").await?;
// cookie "foo=bar" sent automatically

// Response helpers
let resp = client.get("https://httpbin.org/json").await?;
assert!(resp.is_success());
assert_eq!(resp.status(), 200);
let body = resp.text();             // Cow<str> — zero-copy when valid UTF-8
let ct = resp.content_type();       // Option<&str>
let hdr = resp.header("x-custom"); // case-insensitive lookup
let ms = resp.elapsed();            // request duration
```

---

## Architecture

```
webclaw-http (public API — what you import)
  |
  +-- reqwest (forked: exposes rustls_config() for custom TLS)
  |     +-- hyper (forked: passthrough for h2 config)
  |     |     +-- h2 (forked: SETTINGS ordering + pseudo-header order)
  |     +-- rustls (forked: TLS fingerprinting)
  |           - Chrome 146 extension order
  |           - Dummy PSK for Chrome/Edge/Opera
  |           - Safari GREASE + cipher order
  |           - ECH GREASE placeholder
  |
  +-- Browser profiles (Chrome, Firefox, Safari, Edge)
  +-- Header ordering (per-browser HTTP header wire order)
  +-- Bandwidth tracking (atomic, thread-safe, clone-shared)
```

### What we patched and why

| Crate | Upstream | What changed |
|---|---|---|
| **rustls** | 0.23 | ClientHello extension order, dummy PSK binder, GREASE, Safari cipher order |
| **h2** | 0.4 | SETTINGS frame ordering, pseudo-header ordering (`:method :authority :scheme :path`) |
| **hyper** | 1.x | Passthrough for h2 SETTINGS/pseudo config |
| **hyper-util** | 0.1 | Passthrough |
| **reqwest** | 0.13 | Added `rustls_config()` method to bypass `Any` downcast, re-exported `rustls` |

All patches are **additive** behind feature gates. No upstream behavior changed for non-fingerprinting users.

---

## Why `[patch.crates-io]`?

TLS fingerprinting requires changes deep in the dependency chain (rustls ClientHello construction, h2 SETTINGS frame layout). The only way to make reqwest use our patched rustls/h2 is to override them at the workspace level via `[patch.crates-io]`. This is a Cargo-native mechanism — no hacks, no vendoring.

When we publish to crates.io (planned), the patches will ship as standalone crates (`webclaw-rustls`, `webclaw-h2`) and the `[patch.crates-io]` section won't be needed.

---

## Integration Guide

### With an existing reqwest project

If your project already uses reqwest, add the patches to your workspace `Cargo.toml`:

```toml
[dependencies]
webclaw-http = { git = "https://github.com/0xMassi/webclaw-tls" }

[patch.crates-io]
rustls = { git = "https://github.com/0xMassi/webclaw-tls", package = "rustls" }
h2 = { git = "https://github.com/0xMassi/webclaw-tls", package = "h2" }
hyper = { git = "https://github.com/0xMassi/webclaw-tls", package = "hyper" }
hyper-util = { git = "https://github.com/0xMassi/webclaw-tls", package = "hyper-util" }
reqwest = { git = "https://github.com/0xMassi/webclaw-tls", package = "reqwest" }
```

Then replace `reqwest::Client` with `webclaw_http::Client`:

```rust
// Before
let client = reqwest::Client::new();
let resp = client.get("https://example.com").send().await?;

// After
let client = webclaw_http::Client::builder().chrome().build()?;
let resp = client.get("https://example.com").await?;
```

### With webclaw (web scraper)

webclaw already uses webclaw-tls internally. Just use webclaw:

```bash
cargo install webclaw
webclaw https://example.com
```

### RUSTFLAGS

If you use HTTP/3 features, add to `.cargo/config.toml`:

```toml
[build]
rustflags = ["--cfg", "reqwest_unstable"]
```

---

## Tests

36 tests covering TLS fingerprinting, HTTP methods, cookies, redirects, error handling, bandwidth tracking, and concurrent requests:

```bash
cargo test --workspace
```

Key integration tests:
- `chrome_ja4_matches_real_browser` — verifies JA4 + Akamai hash against tls.peet.ws
- `firefox_cipher_and_extension_hash_match` — verifies Firefox JA4 components
- `concurrent_requests` — 10 parallel requests via HTTP/2 multiplexing

---

## License

MIT
