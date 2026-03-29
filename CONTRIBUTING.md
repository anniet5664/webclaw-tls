# Contributing to webclaw-tls

Thanks for your interest in contributing! This project provides browser-grade TLS + HTTP/2 fingerprinting for Rust.

## Getting Started

```bash
git clone https://github.com/0xMassi/webclaw-tls.git
cd webclaw-tls
cargo build
cargo test -p webclaw-http
```

## Project Structure

```
webclaw-tls/
  webclaw-http/       # Public API — what you import
  webclaw-rustls/     # Forked rustls with TLS fingerprinting
  webclaw-h2/         # Forked h2 with HTTP/2 SETTINGS ordering
  webclaw-hyper/      # Forked hyper (passthrough)
  webclaw-hyper-util/ # Forked hyper-util (passthrough)
  webclaw-reqwest/    # Forked reqwest (exposes rustls_config)
  bench/              # Benchmark binary
```

## What to Contribute

**Welcome:**
- New browser profiles (capture from real browsers via [tls.peet.ws](https://tls.peet.ws))
- Bug fixes for specific sites that get blocked
- Performance improvements
- Documentation and examples

**Please discuss first:**
- Changes to the TLS handshake logic (affects fingerprint accuracy)
- New dependencies
- API changes to `webclaw-http`

## Testing

Run our tests before submitting:

```bash
# Unit tests + integration tests
cargo test -p webclaw-http

# Fingerprint verification (requires network)
cargo test -p webclaw-http --test fingerprint
```

Key tests:
- `chrome_ja4_matches_real_browser` — JA4 + Akamai hash vs tls.peet.ws
- `firefox_cipher_and_extension_hash_match` — Firefox JA4 components
- HTTP method tests, cookies, redirects, concurrency

## Pull Requests

1. Fork the repo and create a branch from `main`
2. Run `cargo test -p webclaw-http` and ensure all tests pass
3. If you add a feature, add tests for it
4. Keep PRs focused — one feature or fix per PR
5. Write clear commit messages

## Code Style

- `cargo fmt` before committing
- `cargo clippy` with no warnings
- Follow existing patterns in the codebase

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
