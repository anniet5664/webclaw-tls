//! TLS configuration for browser impersonation.
//!
//! Creates a `rustls::ClientConfig` with `browser_emulation` set to match
//! the target browser's JA4 fingerprint.

use std::sync::{Arc, OnceLock};

// Use reqwest's re-exported rustls to ensure TypeId matches for tls_backend_preconfigured.
use reqwest::rustls;
use rustls::client::{BrowserEmulator, BrowserType, BrowserVersion, EchGreaseConfig, EchMode};
use rustls::crypto::aws_lc_rs;
use rustls::crypto::hpke::HpkePublicKey;
use rustls::{ClientConfig, RootCertStore};

use crate::error::Error;
use crate::profiles::BrowserProfile;

/// Static placeholder ECH public key for GREASE. Same value Chrome uses.
const GREASE_25519_PUBKEY: &[u8] = &[
    0x67, 0x35, 0xCA, 0x50, 0x21, 0xFC, 0x4F, 0xE6, 0x29, 0x3B, 0x31, 0x2C, 0xB5, 0xE0, 0x97, 0xD8,
    0x55, 0x1A, 0x8F, 0x8B, 0xA4, 0x77, 0xAB, 0xFA, 0xBE, 0xA4, 0x53, 0xA3, 0x82, 0x7C, 0x8A, 0x4B,
];

/// Cached ECH GREASE config — created once, reused across all clients.
fn ech_grease_config() -> EchGreaseConfig {
    static CONFIG: OnceLock<EchGreaseConfig> = OnceLock::new();
    CONFIG
        .get_or_init(|| {
            EchGreaseConfig::new(
                aws_lc_rs::hpke::DH_KEM_X25519_HKDF_SHA256_AES_128,
                HpkePublicKey(GREASE_25519_PUBKEY.to_vec()),
            )
        })
        .clone()
}

/// Build a `rustls::ClientConfig` for browser impersonation.
pub fn build_tls_config(profile: &BrowserProfile) -> Result<ClientConfig, Error> {
    let provider = aws_lc_rs::default_provider();

    // Mozilla root CAs + OS native roots. Real browsers use the OS store,
    // which includes cross-signed certs (e.g. Comodo/SSL.com chains) that
    // may not be in Mozilla's static bundle.
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    for cert in rustls_native_certs::load_native_certs().certs {
        root_store.add(cert).ok();
    }

    let is_chrome_based = profile.name.starts_with("Chrome")
        || profile.name.starts_with("Edge")
        || profile.name.starts_with("Opera");
    let is_firefox = profile.name.starts_with("Firefox");
    let needs_ech = is_chrome_based || is_firefox;

    let mut config = if needs_ech {
        ClientConfig::builder_with_provider(Arc::new(provider))
            .with_ech(EchMode::Grease(ech_grease_config()))
            .map_err(|e| Error::Build(format!("ECH config: {e}")))?
            .with_root_certificates(root_store)
            .with_no_client_auth()
    } else {
        ClientConfig::builder_with_provider(Arc::new(provider))
            .with_safe_default_protocol_versions()
            .map_err(|e| Error::Build(format!("TLS config: {e}")))?
            .with_root_certificates(root_store)
            .with_no_client_auth()
    };

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    // Set browser emulation — controls JA4 fingerprint (cipher suites, extensions, PSK)
    let (browser_type, version) = parse_browser_type(profile.name);
    let mut emulator = BrowserEmulator::new(browser_type, version);

    // Set per-browser cipher suites, signature algorithms, and named groups.
    // Without these, rustls uses provider defaults which produce wrong JA4.
    use rustls::crypto::emulation;
    match browser_type {
        BrowserType::Chrome | BrowserType::Edge | BrowserType::Opera => {
            emulator.cipher_suites = Some(emulation::cipher_suites::CHROME.to_vec());
            emulator.signature_algorithms = Some(emulation::signature_algorithms::CHROME.to_vec());
            emulator.named_groups = Some(emulation::named_groups::CHROME.to_vec());
        }
        BrowserType::Firefox => {
            emulator.cipher_suites = Some(emulation::cipher_suites::FIREFOX.to_vec());
            emulator.signature_algorithms = Some(emulation::signature_algorithms::FIREFOX.to_vec());
            emulator.named_groups = Some(emulation::named_groups::FIREFOX.to_vec());
        }
        BrowserType::Safari => {
            emulator.cipher_suites = Some(emulation::cipher_suites::SAFARI.to_vec());
            emulator.signature_algorithms = Some(emulation::signature_algorithms::SAFARI.to_vec());
            emulator.named_groups = Some(emulation::named_groups::SAFARI.to_vec());
        }
    }

    config.browser_emulation = Some(emulator);

    // Cert compression per browser
    if is_chrome_based {
        config.cert_decompressors.retain(|d| {
            matches!(
                d.algorithm(),
                rustls::CertificateCompressionAlgorithm::Brotli
            )
        });
    } else if is_firefox {
        config.cert_decompressors.sort_by(|a, b| {
            let a_zlib = matches!(a.algorithm(), rustls::CertificateCompressionAlgorithm::Zlib);
            let b_zlib = matches!(b.algorithm(), rustls::CertificateCompressionAlgorithm::Zlib);
            b_zlib.cmp(&a_zlib)
        });
    }

    Ok(config)
}

fn parse_browser_type(name: &str) -> (BrowserType, BrowserVersion) {
    let (browser, version_str) = name.split_once('/').unwrap_or((name, "146"));
    let version = BrowserVersion::parse(version_str).unwrap_or(BrowserVersion::new(146, 0, 0));

    let browser_type = match browser {
        "Firefox" => BrowserType::Firefox,
        "Safari" => BrowserType::Safari,
        "Edge" => BrowserType::Edge,
        "Opera" => BrowserType::Opera,
        _ => BrowserType::Chrome,
    };

    (browser_type, version)
}
