<p align="center">
  <a href="https://webclaw.io">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/0xMassi/webclaw/main/.github/banner.png" />
      <img src="https://raw.githubusercontent.com/0xMassi/webclaw/main/.github/banner.png" alt="webclaw" width="700" />
    </picture>
  </a>
</p>

<h3 align="center">
  <strong>ARCHIVED</strong> — webclaw now uses <a href="https://github.com/0x676e67/wreq">wreq</a> for TLS fingerprinting.
</h3>

---

## Status

**This repository is archived.** As of [webclaw v0.3.3](https://github.com/0xMassi/webclaw/releases/tag/v0.3.3), the TLS fingerprinting stack has been replaced with [wreq](https://github.com/0x676e67/wreq) by [@0x676e67](https://github.com/0x676e67), which uses BoringSSL (the same TLS library Chrome ships) and the [http2](https://github.com/0x676e67/http2) crate for HTTP/2 fingerprinting.

### Why we switched

This repo contained surgical patches to rustls, h2, hyper, hyper-util, and reqwest to achieve browser-grade TLS + HTTP/2 fingerprinting. While the approach worked for many sites, it had fundamental limitations:

- **TLS compatibility gaps** — our patched rustls rejected valid server configurations (e.g. [Vontobel](https://github.com/0xMassi/webclaw/issues/8) returned `IllegalParameter`). BoringSSL handles these correctly because it's the actual TLS library Chrome uses.
- **Solo maintenance burden** — keeping 5 forked crates in sync with upstream security patches is unsustainable for a single maintainer.
- **Attribution** — the HTTP/2 fingerprinting concepts (SETTINGS frame ordering, pseudo-header ordering) in our h2 fork were derived from work pioneered by [@0x676e67](https://github.com/0x676e67). Using his maintained library directly is both technically superior and the right thing to do.

### What to use instead

If you need browser TLS fingerprinting in Rust:

- **[wreq](https://github.com/0x676e67/wreq)** — BoringSSL, 60+ browser profiles, HTTP/2 fingerprinting via [http2](https://github.com/0x676e67/http2). Apache-2.0 license.
- **[impit](https://github.com/apify/impit)** — patched rustls, Node.js + Python bindings, by Apify.

For web extraction with built-in fingerprinting, use [webclaw](https://github.com/0xMassi/webclaw) directly:

```bash
brew tap 0xMassi/webclaw && brew install webclaw
```

### Acknowledgments

TLS and HTTP/2 browser fingerprinting in the Rust ecosystem was pioneered by [@0x676e67](https://github.com/0x676e67). His [http2](https://github.com/0x676e67/http2) crate introduced configurable SETTINGS frame ordering and pseudo-header ordering, and [wreq](https://github.com/0x676e67/wreq) brought BoringSSL-based TLS fingerprinting to Rust. This work powers webclaw and many other projects in the scraping community.

## License

[MIT](LICENSE)
