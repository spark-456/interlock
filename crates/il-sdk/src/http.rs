//! SSRF-safe outbound HTTP (PRD §6 Security): blocks private, loopback, link-local and other
//! internal ranges on every DNS resolution and every redirect hop.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use reqwest::{Method, RequestBuilder, redirect};
use url::Url;

/// HTTP errors raised before a request is sent.
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    #[error("blocked destination: {0}")]
    Blocked(String),
    #[error("http: {0}")]
    Client(String),
}

/// A reqwest client that refuses internal destinations. Cheap to clone.
#[derive(Clone)]
pub struct SafeHttp {
    client: reqwest::Client,
    allow_private: bool,
}

impl std::fmt::Debug for SafeHttp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SafeHttp").field("allow_private", &self.allow_private).finish()
    }
}

impl SafeHttp {
    /// The production client.
    pub fn new() -> Result<Self, HttpError> {
        Self::build(false)
    }

    /// A client that allows loopback/private hosts, for tests against local mock servers only.
    pub fn permissive_for_tests() -> Self {
        Self::build(true).unwrap_or_else(|_| Self { client: reqwest::Client::new(), allow_private: true })
    }

    fn build(allow_private: bool) -> Result<Self, HttpError> {
        let mut builder = reqwest::Client::builder()
            .user_agent(concat!("Interlock/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(120))
            .redirect(redirect::Policy::custom(move |attempt| {
                if attempt.previous().len() >= 5 {
                    return attempt.error("too many redirects");
                }
                match check_url(attempt.url(), allow_private) {
                    Ok(()) => attempt.follow(),
                    Err(e) => attempt.error(e.to_string()),
                }
            }));
        if !allow_private {
            builder = builder.dns_resolver(Arc::new(SafeResolver));
        }
        let client = builder.build().map_err(|e| HttpError::Client(e.to_string()))?;
        Ok(Self { client, allow_private })
    }

    /// Starts a request after checking the URL.
    pub fn request(&self, method: Method, url: &str) -> Result<RequestBuilder, HttpError> {
        let parsed = Url::parse(url).map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        check_url(&parsed, self.allow_private)?;
        Ok(self.client.request(method, parsed))
    }

    /// GET shorthand.
    pub fn get(&self, url: &str) -> Result<RequestBuilder, HttpError> {
        self.request(Method::GET, url)
    }

    /// POST shorthand.
    pub fn post(&self, url: &str) -> Result<RequestBuilder, HttpError> {
        self.request(Method::POST, url)
    }

    /// Checks a URL before handing it to a remote fetching service.
    pub async fn validate_destination(&self, url: &str) -> Result<(), HttpError> {
        let parsed = Url::parse(url).map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        check_url(&parsed, self.allow_private)?;
        if self.allow_private {
            return Ok(());
        }
        if let Some(url::Host::Domain(host)) = parsed.host() {
            let port = parsed.port_or_known_default().unwrap_or(443);
            let addrs: Vec<_> =
                tokio::net::lookup_host((host, port)).await.map_err(|e| HttpError::Client(e.to_string()))?.collect();
            if addrs.is_empty() || addrs.iter().any(|addr| is_blocked_ip(addr.ip())) {
                return Err(HttpError::Blocked(host.into()));
            }
        }
        Ok(())
    }
}

fn check_url(url: &Url, allow_private: bool) -> Result<(), HttpError> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err(HttpError::Blocked(format!("scheme {}", url.scheme())));
    }
    if allow_private {
        return Ok(());
    }
    match url.host() {
        Some(url::Host::Ipv4(ip)) if is_blocked_ip(IpAddr::V4(ip)) => Err(HttpError::Blocked(ip.to_string())),
        Some(url::Host::Ipv6(ip)) if is_blocked_ip(IpAddr::V6(ip)) => Err(HttpError::Blocked(ip.to_string())),
        Some(url::Host::Domain(d)) if d.eq_ignore_ascii_case("localhost") || d.ends_with(".localhost") => {
            Err(HttpError::Blocked(d.to_string()))
        }
        None => Err(HttpError::InvalidUrl("missing host".into())),
        _ => Ok(()),
    }
}

/// True for addresses outbound requests must never reach.
pub fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_v4(v4),
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_blocked_v4(v4);
            }
            is_blocked_v6(v6)
        }
    }
}

fn is_blocked_v4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.is_multicast()
        || o[0] == 0
        || (o[0] == 100 && (64..128).contains(&o[1])) // shared address space
        || (o[0] == 198 && (o[1] == 18 || o[1] == 19)) // benchmarking
        || o[0] >= 240 // reserved
}

fn is_blocked_v6(ip: Ipv6Addr) -> bool {
    let seg0 = ip.segments()[0];
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (seg0 & 0xfe00) == 0xfc00 // unique local
        || (seg0 & 0xffc0) == 0xfe80 // link local
        || (seg0 == 0x2001 && ip.segments()[1] == 0x0db8) // documentation
        || (seg0 == 0x0064 && ip.segments()[1] == 0xff9b) // NAT64 (may embed private v4)
}

struct SafeResolver;

impl Resolve for SafeResolver {
    fn resolve(&self, name: Name) -> Resolving {
        Box::pin(async move {
            let host = name.as_str().to_string();
            let addrs: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), 0)).await?.collect();
            if addrs.is_empty() || addrs.iter().any(|a| is_blocked_ip(a.ip())) {
                return Err(Box::new(HttpError::Blocked(host)) as Box<dyn std::error::Error + Send + Sync>);
            }
            Ok(Box::new(addrs.into_iter()) as Addrs)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_internal_ranges() {
        for ip in [
            "127.0.0.1",
            "10.1.2.3",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "0.0.0.0",
            "100.64.0.1",
            "::1",
            "fe80::1",
            "fd00::1",
            "::ffff:10.0.0.1",
        ] {
            assert!(is_blocked_ip(ip.parse().unwrap()), "{ip} should be blocked");
        }
        for ip in ["8.8.8.8", "1.1.1.1", "2606:4700:4700::1111"] {
            assert!(!is_blocked_ip(ip.parse().unwrap()), "{ip} should be allowed");
        }
    }

    #[test]
    fn rejects_bad_urls_before_sending() {
        let http = SafeHttp::new().unwrap();
        assert!(matches!(http.get("http://127.0.0.1:8080/"), Err(HttpError::Blocked(_))));
        assert!(matches!(http.get("http://localhost/"), Err(HttpError::Blocked(_))));
        assert!(matches!(http.get("file:///etc/passwd"), Err(HttpError::Blocked(_))));
        assert!(matches!(http.get("http://[::1]/"), Err(HttpError::Blocked(_))));
        assert!(http.get("https://example.com/").is_ok());
    }

    #[tokio::test]
    async fn resolver_blocks_names_pointing_inside() {
        let addrs = SafeResolver.resolve("localhost".parse().unwrap()).await;
        assert!(addrs.is_err());
    }

    #[test]
    fn permissive_client_allows_loopback() {
        assert!(SafeHttp::permissive_for_tests().get("http://127.0.0.1:1/").is_ok());
    }

    #[tokio::test]
    async fn rejects_internal_remote_fetch_destinations() {
        let http = SafeHttp::new().unwrap();
        assert!(matches!(
            http.validate_destination("http://169.254.169.254/latest/meta-data/").await,
            Err(HttpError::Blocked(_))
        ));
        assert!(matches!(http.validate_destination("http://localhost/private").await, Err(HttpError::Blocked(_))));
    }
}
