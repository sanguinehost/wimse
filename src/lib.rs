//! `spiffe-id` — a typed, `no_std`-friendly parser/validator for
//! [SPIFFE-ID](https://github.com/spiffe/spiffe/blob/main/standards/SPIFFE-ID.md)
//! URIs and [WIMSE](https://datatracker.ietf.org/wg/wimse/about/) workload
//! identifiers.
//!
//! Implements the contract in `docs/specs/iter-18-spiffe-id.md` from
//! [`sanguinehost/ferrousgate`](https://github.com/sanguinehost/ferrousgate).
//!
//! The crate parses with `core` + `alloc` only — no I/O, no crypto — so it can
//! be compiled to `wasm32-unknown-unknown` and used in the browser.
//!
//! ```
//! use spiffe_id::SpiffeId;
//!
//! let id = SpiffeId::parse("spiffe://twn.network/ns/prod/sa/agent-1").unwrap();
//! assert_eq!(id.trust_domain(), "twn.network");
//! assert_eq!(id.path(), "ns/prod/sa/agent-1");
//! let wl = id.as_wimse_workload().unwrap();
//! assert_eq!(wl.namespace, "prod");
//! assert_eq!(wl.service_account, "agent-1");
//! assert!(wl.extra.is_empty());
//! assert_eq!(id.as_uri(), "spiffe://twn.network/ns/prod/sa/agent-1");
//! ```

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// The SPIFFE-ID spec caps the full URI at 2048 bytes.
const MAX_URI_LEN: usize = 2048;
/// The SPIFFE-ID spec caps the trust domain at 255 bytes.
const MAX_TRUST_DOMAIN_LEN: usize = 255;

/// Errors returned by [`SpiffeId::parse`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The scheme was not `spiffe`.
    BadScheme,
    /// The URI carried userinfo, a port, a query, or a fragment.
    DisallowedComponent(&'static str),
    /// The trust domain was empty, too long (> 255 bytes), or contained a
    /// disallowed character.
    BadTrustDomain(&'static str),
    /// A path segment was empty (`//` or trailing `/`), too long, or contained
    /// a disallowed character / bad percent-encoding.
    BadPath(&'static str),
    /// The full URI exceeded the 2048-byte cap.
    TooLong,
    /// Generic syntax error — malformed URI.
    Malformed,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::BadScheme => f.write_str("SPIFFE ID scheme must be `spiffe`"),
            ParseError::DisallowedComponent(c) => write!(f, "SPIFFE ID must not contain {c}"),
            ParseError::BadTrustDomain(why) => write!(f, "invalid trust domain: {why}"),
            ParseError::BadPath(why) => write!(f, "invalid path: {why}"),
            ParseError::TooLong => write!(f, "SPIFFE ID URI exceeds {MAX_URI_LEN} bytes"),
            ParseError::Malformed => f.write_str("malformed SPIFFE ID URI"),
        }
    }
}

// `core::error::Error` is stable since Rust 1.81.
impl core::error::Error for ParseError {}

/// A validated SPIFFE ID.
///
/// Construct via [`SpiffeId::parse`]. The stored form is always canonical:
/// the trust domain is lowercase, the path is percent-decoded, and there is no
/// trailing slash.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpiffeId {
    trust_domain: String,
    /// Path *without* the leading slash. Empty for a trust-domain-only ID
    /// (`spiffe://td` — valid per the spec, used for bundle identifiers).
    path: String,
}

impl SpiffeId {
    /// Parse + validate a SPIFFE ID URI.
    ///
    /// # Errors
    /// A [`ParseError`] variant for any spec violation.
    pub fn parse(s: &str) -> Result<SpiffeId, ParseError> {
        // Behavior 7: full URI length cap.
        if s.len() > MAX_URI_LEN {
            return Err(ParseError::TooLong);
        }

        // Behavior 3: scheme must be `spiffe` (case-insensitive accept).
        let rest = split_scheme(s)?;

        // SPIFFE IDs are always `scheme://authority[/path]` — the `//` is
        // mandatory (there is always an authority component).
        let after_slashes = rest.strip_prefix("//").ok_or(ParseError::Malformed)?;

        // The authority runs up to the first `/`, `?`, or `#`.
        let (authority, tail) = split_at_any(after_slashes, &['/', '?', '#']);

        // Behavior 4: no userinfo, no port.
        if authority.contains('@') {
            return Err(ParseError::DisallowedComponent("userinfo"));
        }
        // There is no userinfo in a SPIFFE authority, so any `:` is a port
        // separator. (An IPv6 literal host would also carry `:` but is not a
        // valid trust domain regardless.)
        if authority.contains(':') {
            return Err(ParseError::DisallowedComponent("port"));
        }

        // Behavior 5: trust domain validation (after lowercasing).
        let trust_domain = validate_trust_domain(authority)?;

        // What follows the authority is one of:
        //   - nothing                  → trust-domain-only ID
        //   - "/" + path [+ ?…/#…]     → ID with a path
        //   - "?…" / "#…"              → disallowed component
        let path = if tail.is_empty() {
            String::new()
        } else if let Some(after_slash) = tail.strip_prefix('/') {
            // Isolate the path body from any query/fragment that follows it.
            let (path_body, leftover) = split_at_any(after_slash, &['?', '#']);
            if !leftover.is_empty() {
                return Err(disallowed_for_delim(leftover));
            }
            // Behavior 6.2: a path may not end with `/`. `spiffe://td/` lands
            // here with an empty `path_body`; both that and a real trailing
            // slash (`spiffe://td/a/`) are rejected by `validate_path`.
            validate_path(path_body)?
        } else {
            // `tail` started with `?` or `#`.
            return Err(disallowed_for_delim(tail));
        };

        Ok(SpiffeId { trust_domain, path })
    }

    /// The trust domain, lowercase, without the `spiffe://` prefix.
    #[must_use]
    pub fn trust_domain(&self) -> &str {
        &self.trust_domain
    }

    /// The path, percent-decoded, without the leading slash. Empty for a
    /// trust-domain-only ID.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The path split on `/` into segments. Empty `Vec` for a trust-domain-only
    /// ID.
    #[must_use]
    pub fn path_segments(&self) -> Vec<&str> {
        if self.path.is_empty() {
            Vec::new()
        } else {
            self.path.split('/').collect()
        }
    }

    /// Whether this ID belongs to the given trust domain (exact,
    /// case-insensitive).
    #[must_use]
    pub fn in_trust_domain(&self, td: &str) -> bool {
        eq_ignore_ascii_case(&self.trust_domain, td)
    }

    /// Render back to the canonical `spiffe://…` URI string.
    #[must_use]
    pub fn as_uri(&self) -> String {
        let mut out = String::with_capacity(9 + self.trust_domain.len() + 1 + self.path.len());
        out.push_str("spiffe://");
        out.push_str(&self.trust_domain);
        if !self.path.is_empty() {
            out.push('/');
            out.push_str(&self.path);
        }
        out
    }

    /// Attempt to interpret the path as a WIMSE workload identifier.
    ///
    /// Returns `Some` when the path matches the
    /// `ns/<namespace>/sa/<service-account>[/<extra>...]` convention with
    /// `<namespace>` and `<service-account>` non-empty; `None` otherwise (the
    /// ID is still valid — it just isn't a WIMSE workload ID). Never errors.
    #[must_use]
    pub fn as_wimse_workload(&self) -> Option<WimseWorkloadId<'_>> {
        let segs = self.path_segments();
        // Need at least: ns, <namespace>, sa, <service-account>.
        if segs.len() < 4 {
            return None;
        }
        if segs[0] != "ns" || segs[2] != "sa" {
            return None;
        }
        let namespace = segs[1];
        let service_account = segs[3];
        if namespace.is_empty() || service_account.is_empty() {
            return None;
        }
        let extra = segs[4..].to_vec();
        Some(WimseWorkloadId {
            namespace,
            service_account,
            extra,
        })
    }
}

/// Structured view of a WIMSE workload identifier embedded in a SPIFFE ID path.
/// Borrows from the parent [`SpiffeId`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WimseWorkloadId<'a> {
    /// The `<namespace>` segment.
    pub namespace: &'a str,
    /// The `<service-account>` segment.
    pub service_account: &'a str,
    /// Any path segments after `sa/<service-account>`. Empty if none.
    pub extra: Vec<&'a str>,
}

impl fmt::Display for SpiffeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Behavior 10: render as_uri().
        f.write_str("spiffe://")?;
        f.write_str(&self.trust_domain)?;
        if !self.path.is_empty() {
            f.write_str("/")?;
            f.write_str(&self.path)?;
        }
        Ok(())
    }
}

impl core::str::FromStr for SpiffeId {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Behavior 10: FromStr delegates to parse().
        SpiffeId::parse(s)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Splits off the `scheme:` prefix, validating it is `spiffe` (case-insensitive).
/// Returns the remainder *after* the colon.
fn split_scheme(s: &str) -> Result<&str, ParseError> {
    // RFC 3986 scheme = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." ), then ":".
    let colon = s.find(':').ok_or(ParseError::Malformed)?;
    let scheme = &s[..colon];
    if scheme.is_empty() {
        return Err(ParseError::BadScheme);
    }
    if !eq_ignore_ascii_case(scheme, "spiffe") {
        return Err(ParseError::BadScheme);
    }
    Ok(&s[colon + 1..])
}

/// Splits `s` at the first occurrence of any char in `delims`. Returns
/// `(before, from_delim_onwards)` — the second element *includes* the
/// delimiter, or is empty if no delimiter was found.
fn split_at_any<'a>(s: &'a str, delims: &[char]) -> (&'a str, &'a str) {
    match s.find(|c| delims.contains(&c)) {
        Some(i) => (&s[..i], &s[i..]),
        None => (s, ""),
    }
}

/// Maps a leftover `?…`/`#…` tail to the right `DisallowedComponent`.
fn disallowed_for_delim(tail: &str) -> ParseError {
    match tail.as_bytes().first() {
        Some(b'?') => ParseError::DisallowedComponent("query"),
        Some(b'#') => ParseError::DisallowedComponent("fragment"),
        _ => ParseError::Malformed,
    }
}

/// Validates the trust domain per Behavior 5, returning the canonical
/// (lowercased) form.
fn validate_trust_domain(raw: &str) -> Result<String, ParseError> {
    if raw.is_empty() {
        return Err(ParseError::BadTrustDomain("trust domain must not be empty"));
    }
    if raw.len() > MAX_TRUST_DOMAIN_LEN {
        return Err(ParseError::BadTrustDomain("trust domain exceeds 255 bytes"));
    }
    let lowered = ascii_lowercase(raw);
    for &b in lowered.as_bytes() {
        if !is_trust_domain_byte(b) {
            return Err(ParseError::BadTrustDomain(
                "trust domain may only contain [a-z0-9._-]",
            ));
        }
    }
    Ok(lowered)
}

/// Validates the path body (without leading slash) per Behavior 6, returning
/// the canonical (percent-decoded) form.
fn validate_path(body: &str) -> Result<String, ParseError> {
    // `body` is the path with the leading slash already stripped. An empty
    // `body` means the original was `spiffe://td/` — a trailing slash with an
    // empty path, which is rejected.
    if body.is_empty() {
        return Err(ParseError::BadPath("path must not be empty"));
    }
    // Behavior 6.2: no trailing `/`.
    if body.ends_with('/') {
        return Err(ParseError::BadPath("path must not end with `/`"));
    }
    let mut decoded_path = String::with_capacity(body.len());
    let mut first = true;
    for seg in body.split('/') {
        if seg.is_empty() {
            return Err(ParseError::BadPath("path must not contain empty segments"));
        }
        let decoded_seg = percent_decode_segment(seg)?;
        if decoded_seg.is_empty() {
            // e.g. a segment that was purely... there's no encoding that
            // produces an empty string, but be defensive.
            return Err(ParseError::BadPath("path segment decoded to empty"));
        }
        for &b in decoded_seg.as_bytes() {
            if !is_path_byte(b) {
                return Err(ParseError::BadPath(
                    "path segment may only contain [A-Za-z0-9._-]",
                ));
            }
        }
        if !first {
            decoded_path.push('/');
        }
        decoded_path.push_str(&decoded_seg);
        first = false;
    }
    Ok(decoded_path)
}

/// Percent-decodes one path segment. Rejects malformed `%XX` sequences with
/// [`ParseError::BadPath`]. Bytes that decode are kept as raw bytes; the result
/// must still be valid UTF-8 (which, for the SPIFFE reserved set, it always is
/// — but a `%C3%A9` would decode to `é`, valid UTF-8 yet a disallowed char,
/// caught by the byte check in `validate_path`).
fn percent_decode_segment(seg: &str) -> Result<String, ParseError> {
    let bytes = seg.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'%' {
            if i + 2 >= bytes.len() {
                return Err(ParseError::BadPath("truncated percent-encoding"));
            }
            let hi =
                hex_val(bytes[i + 1]).ok_or(ParseError::BadPath("invalid percent-encoding"))?;
            let lo =
                hex_val(bytes[i + 2]).ok_or(ParseError::BadPath("invalid percent-encoding"))?;
            out.push((hi << 4) | lo);
            i += 3;
        } else {
            out.push(b);
            i += 1;
        }
    }
    String::from_utf8(out).map_err(|_| ParseError::BadPath("path segment is not valid UTF-8"))
}

#[inline]
fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[inline]
fn is_trust_domain_byte(b: u8) -> bool {
    matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_')
}

#[inline]
fn is_path_byte(b: u8) -> bool {
    matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_')
}

#[inline]
fn eq_ignore_ascii_case(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .all(|(x, y)| x.eq_ignore_ascii_case(&y))
}

fn ascii_lowercase(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        out.push(b.to_ascii_lowercase() as char);
    }
    out
}

// ---------------------------------------------------------------------------
// serde
// ---------------------------------------------------------------------------

#[cfg(feature = "serde")]
mod serde_impls {
    use super::SpiffeId;
    use alloc::string::String;
    use core::fmt;
    use serde::de::{self, Visitor};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for SpiffeId {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // Behavior 11: serialize as the URI string.
            serializer.collect_str(self)
        }
    }

    impl<'de> Deserialize<'de> for SpiffeId {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct SpiffeIdVisitor;

            impl Visitor<'_> for SpiffeIdVisitor {
                type Value = SpiffeId;

                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("a SPIFFE ID URI string (spiffe://<trust-domain>/<path>)")
                }

                fn visit_str<E>(self, v: &str) -> Result<SpiffeId, E>
                where
                    E: de::Error,
                {
                    // Behavior 11: deserialize via parse(), serde error on failure.
                    SpiffeId::parse(v).map_err(de::Error::custom)
                }

                fn visit_string<E>(self, v: String) -> Result<SpiffeId, E>
                where
                    E: de::Error,
                {
                    self.visit_str(&v)
                }
            }

            deserializer.deserialize_str(SpiffeIdVisitor)
        }
    }
}
