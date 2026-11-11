//! The `Content-Security-Policy` every response carries.

use topcoat::router::header::{HeaderValue, InvalidHeaderValue};

/// The `style` attribute `topcoat-icon` writes on every icon, `vertical-align:
/// -0.125em`, allowed by its SHA-256 hash and by no other means, since the
/// crate gives no way to leave it out. An upgrade that changes it shows as a
/// CSP violation on any page with an icon.
const ICON_STYLE_HASH: &str = "'sha256-4AXVd4Ue0A6d/wfNkdhs1EwnaIBGk3lC7Q5dkpfGJEA='";

/// The policy of this application, built once from its configuration.
///
/// Scripts come only from this origin; Topcoat's browser runtime compiles the
/// expressions of its shards with `new Function`, so scripts also need
/// `'unsafe-eval'`. Styles come only from the application's stylesheet, and no
/// `style` attribute applies but the icons'. The browser uploads files straight
/// to the object storage, which is the only other origin it connects to.
#[derive(Debug, Clone)]
pub struct ContentSecurityPolicy(HeaderValue);

impl ContentSecurityPolicy {
    /// Builds the policy for uploads to `storage_url`, an origin.
    ///
    /// # Errors
    ///
    /// Fails if `storage_url` cannot appear in a header.
    pub fn new(storage_url: &str) -> Result<Self, InvalidHeaderValue> {
        let policy = [
            "default-src 'self'".to_owned(),
            "script-src 'self' 'unsafe-eval'".to_owned(),
            "style-src 'self'".to_owned(),
            format!("style-src-attr 'unsafe-hashes' {ICON_STYLE_HASH}"),
            "img-src 'self' data:".to_owned(),
            "font-src 'self'".to_owned(),
            format!("connect-src 'self' {storage_url}"),
            "object-src 'none'".to_owned(),
            "base-uri 'self'".to_owned(),
            "form-action 'self'".to_owned(),
            "frame-ancestors 'none'".to_owned(),
        ]
        .join("; ");
        HeaderValue::from_str(&policy).map(Self)
    }

    /// The value of the `Content-Security-Policy` header.
    #[must_use]
    pub fn header(&self) -> &HeaderValue {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::ContentSecurityPolicy;

    fn policy() -> String {
        ContentSecurityPolicy::new("https://files.example.com")
            .unwrap()
            .header()
            .to_str()
            .unwrap()
            .to_owned()
    }

    #[test]
    fn uploads_may_reach_the_storage_and_nothing_else_outside() {
        assert!(policy().contains("connect-src 'self' https://files.example.com;"));
        assert!(policy().contains("default-src 'self';"));
    }

    #[test]
    fn no_inline_script_or_style_runs() {
        let policy = policy();
        assert!(!policy.contains("'unsafe-inline'"));
        assert!(policy.contains("script-src 'self' 'unsafe-eval';"));
        assert!(policy.contains("style-src 'self';"));
    }

    #[test]
    fn no_other_site_frames_or_retargets_the_pages() {
        let policy = policy();
        assert!(policy.contains("frame-ancestors 'none'"));
        assert!(policy.contains("base-uri 'self';"));
        assert!(policy.contains("form-action 'self';"));
        assert!(policy.contains("object-src 'none';"));
    }
}
