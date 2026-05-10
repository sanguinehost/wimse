//! Tests for the `spiffe-id` crate, per `docs/specs/iter-18-spiffe-id.md`
//! (`sanguinehost/ferrousgate`) `## Test plan`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use core::str::FromStr;
use spiffe_id::{ParseError, SpiffeId};

// --- MUST: valid ID with a path -------------------------------------------

#[test]
fn valid_id_with_path() {
    let id = SpiffeId::parse("spiffe://twn.network/agent/orchestrator").unwrap();
    assert_eq!(id.trust_domain(), "twn.network");
    assert_eq!(id.path(), "agent/orchestrator");
    assert_eq!(id.path_segments(), vec!["agent", "orchestrator"]);
}

// --- MUST: valid trust-domain-only ID -------------------------------------

#[test]
fn valid_trust_domain_only() {
    let id = SpiffeId::parse("spiffe://twn.network").unwrap();
    assert_eq!(id.trust_domain(), "twn.network");
    assert_eq!(id.path(), "");
    assert!(id.path_segments().is_empty());
    assert_eq!(id.as_uri(), "spiffe://twn.network");
}

// --- MUST: uppercase trust domain is canonicalised ------------------------

#[test]
fn uppercase_trust_domain_canonicalised() {
    let id = SpiffeId::parse("spiffe://TWN.Network/x").unwrap();
    assert_eq!(id.trust_domain(), "twn.network");
    assert_eq!(id.as_uri(), "spiffe://twn.network/x");
}

#[test]
fn uppercase_scheme_accepted() {
    let id = SpiffeId::parse("SPIFFE://twn.network/x").unwrap();
    assert_eq!(id.as_uri(), "spiffe://twn.network/x");
}

// --- MUST: round-trip table -----------------------------------------------

#[test]
fn round_trip_table() {
    let cases = [
        "spiffe://twn.network",
        "spiffe://twn.network/x",
        "spiffe://twn.network/agent/orchestrator",
        "spiffe://twn.network/ns/prod/sa/agent-1",
        "spiffe://twn.network/ns/prod/sa/agent-1/region/au",
        "spiffe://example.org/a.b_c-d",
        "spiffe://a",
        "spiffe://sub.domain.example.com/path/with/many/segments",
    ];
    for c in cases {
        let id = SpiffeId::parse(c).unwrap();
        let reparsed = SpiffeId::parse(&id.as_uri()).unwrap();
        assert_eq!(reparsed, id, "round-trip failed for {c}");
        assert_eq!(id.as_uri(), c, "as_uri not stable for {c}");
    }
}

#[test]
fn round_trip_after_canonicalisation() {
    // Inputs that change on canonicalisation must still round-trip from as_uri.
    let id = SpiffeId::parse("spiffe://TWN.NETWORK/a%2Db").unwrap();
    let again = SpiffeId::parse(&id.as_uri()).unwrap();
    assert_eq!(again, id);
    assert_eq!(id.as_uri(), "spiffe://twn.network/a-b");
}

// --- MUST: reject bad scheme ----------------------------------------------

#[test]
fn reject_bad_scheme() {
    assert_eq!(
        SpiffeId::parse("https://twn.network/x"),
        Err(ParseError::BadScheme)
    );
}

// --- MUST: reject userinfo ------------------------------------------------

#[test]
fn reject_userinfo() {
    assert_eq!(
        SpiffeId::parse("spiffe://user@twn.network/x"),
        Err(ParseError::DisallowedComponent("userinfo"))
    );
}

// --- MUST: reject port ----------------------------------------------------

#[test]
fn reject_port() {
    assert_eq!(
        SpiffeId::parse("spiffe://twn.network:8080/x"),
        Err(ParseError::DisallowedComponent("port"))
    );
}

// --- MUST: reject query ---------------------------------------------------

#[test]
fn reject_query() {
    assert_eq!(
        SpiffeId::parse("spiffe://twn.network/x?y=1"),
        Err(ParseError::DisallowedComponent("query"))
    );
}

#[test]
fn reject_query_without_path() {
    assert_eq!(
        SpiffeId::parse("spiffe://twn.network?y=1"),
        Err(ParseError::DisallowedComponent("query"))
    );
}

// --- MUST: reject fragment ------------------------------------------------

#[test]
fn reject_fragment() {
    assert_eq!(
        SpiffeId::parse("spiffe://twn.network/x#y"),
        Err(ParseError::DisallowedComponent("fragment"))
    );
}

#[test]
fn reject_fragment_without_path() {
    assert_eq!(
        SpiffeId::parse("spiffe://twn.network#y"),
        Err(ParseError::DisallowedComponent("fragment"))
    );
}

// --- MUST: reject empty trust domain --------------------------------------

#[test]
fn reject_empty_trust_domain() {
    match SpiffeId::parse("spiffe:///x") {
        Err(ParseError::BadTrustDomain(_)) => {}
        other => panic!("expected BadTrustDomain, got {other:?}"),
    }
}

#[test]
fn reject_no_authority() {
    // `spiffe:` with no `//` is malformed.
    assert_eq!(
        SpiffeId::parse("spiffe:twn.network/x"),
        Err(ParseError::Malformed)
    );
}

// --- MUST: reject too-long trust domain (256 bytes) -----------------------

#[test]
fn reject_too_long_trust_domain() {
    let td = "a".repeat(256);
    let uri = format!("spiffe://{td}/x");
    match SpiffeId::parse(&uri) {
        Err(ParseError::BadTrustDomain(_)) => {}
        other => panic!("expected BadTrustDomain, got {other:?}"),
    }
    // 255 bytes is fine.
    let td_ok = "a".repeat(255);
    let uri_ok = format!("spiffe://{td_ok}/x");
    assert!(SpiffeId::parse(&uri_ok).is_ok());
}

// --- MUST: reject bad trust-domain char -----------------------------------

#[test]
fn reject_bad_trust_domain_char() {
    match SpiffeId::parse("spiffe://twn network/x") {
        Err(ParseError::BadTrustDomain(_)) => {}
        other => panic!("expected BadTrustDomain, got {other:?}"),
    }
}

#[test]
fn reject_bad_trust_domain_char_slash_encoded() {
    // A `+` is not in the SPIFFE trust-domain set.
    match SpiffeId::parse("spiffe://twn+network/x") {
        Err(ParseError::BadTrustDomain(_)) => {}
        other => panic!("expected BadTrustDomain, got {other:?}"),
    }
}

// --- MUST: reject trailing slash ------------------------------------------

#[test]
fn reject_trailing_slash() {
    match SpiffeId::parse("spiffe://twn.network/x/") {
        Err(ParseError::BadPath(_)) => {}
        other => panic!("expected BadPath, got {other:?}"),
    }
    // Trailing slash on a trust-domain-only-ish URI.
    match SpiffeId::parse("spiffe://twn.network/") {
        Err(ParseError::BadPath(_)) => {}
        other => panic!("expected BadPath, got {other:?}"),
    }
}

// --- MUST: reject empty segment -------------------------------------------

#[test]
fn reject_empty_segment() {
    match SpiffeId::parse("spiffe://twn.network/x//y") {
        Err(ParseError::BadPath(_)) => {}
        other => panic!("expected BadPath, got {other:?}"),
    }
}

// --- MUST: reject bad path char -------------------------------------------

#[test]
fn reject_bad_path_char() {
    match SpiffeId::parse("spiffe://twn.network/x y") {
        Err(ParseError::BadPath(_)) => {}
        other => panic!("expected BadPath, got {other:?}"),
    }
}

// --- MUST: percent-decode path --------------------------------------------

#[test]
fn percent_decode_allowed_char() {
    let id = SpiffeId::parse("spiffe://twn.network/a%2Db").unwrap();
    assert_eq!(id.path(), "a-b");
    assert_eq!(id.path_segments(), vec!["a-b"]);
    assert_eq!(id.as_uri(), "spiffe://twn.network/a-b");
}

#[test]
fn percent_decode_lowercase_hex() {
    let id = SpiffeId::parse("spiffe://twn.network/a%2db").unwrap();
    assert_eq!(id.path(), "a-b");
}

#[test]
fn percent_decode_disallowed_char_rejected() {
    // %20 = space — well-formed encoding, but decodes to a disallowed char.
    match SpiffeId::parse("spiffe://twn.network/a%20b") {
        Err(ParseError::BadPath(_)) => {}
        other => panic!("expected BadPath, got {other:?}"),
    }
}

#[test]
fn percent_decode_malformed_rejected() {
    // Truncated percent-encoding.
    match SpiffeId::parse("spiffe://twn.network/a%2") {
        Err(ParseError::BadPath(_)) => {}
        other => panic!("expected BadPath, got {other:?}"),
    }
    // Non-hex digits.
    match SpiffeId::parse("spiffe://twn.network/a%2Zb") {
        Err(ParseError::BadPath(_)) => {}
        other => panic!("expected BadPath, got {other:?}"),
    }
    // %2F = `/` — decoding it would create an empty/extra segment; the
    // decoded `/` is not in the allowed segment set, so it is rejected.
    match SpiffeId::parse("spiffe://twn.network/a%2Fb") {
        Err(ParseError::BadPath(_)) => {}
        other => panic!("expected BadPath, got {other:?}"),
    }
}

// --- MUST: reject too-long URI (2049 bytes) -------------------------------

#[test]
fn reject_too_long_uri() {
    // "spiffe://twn.network/" is 21 bytes; pad the path to total 2049.
    let prefix = "spiffe://twn.network/";
    let pad = 2049 - prefix.len();
    let uri = format!("{prefix}{}", "a".repeat(pad));
    assert_eq!(uri.len(), 2049);
    assert_eq!(SpiffeId::parse(&uri), Err(ParseError::TooLong));
    // 2048 bytes is fine.
    let uri_ok = format!("{prefix}{}", "a".repeat(pad - 1));
    assert_eq!(uri_ok.len(), 2048);
    assert!(SpiffeId::parse(&uri_ok).is_ok());
}

// --- MUST: as_wimse_workload ----------------------------------------------

#[test]
fn wimse_workload_basic() {
    let id = SpiffeId::parse("spiffe://twn.network/ns/prod/sa/agent-1").unwrap();
    let wl = id.as_wimse_workload().expect("should be a WIMSE workload");
    assert_eq!(wl.namespace, "prod");
    assert_eq!(wl.service_account, "agent-1");
    assert!(wl.extra.is_empty());
}

#[test]
fn wimse_workload_with_extra() {
    let id = SpiffeId::parse("spiffe://twn.network/ns/prod/sa/agent-1/region/au").unwrap();
    let wl = id.as_wimse_workload().expect("should be a WIMSE workload");
    assert_eq!(wl.namespace, "prod");
    assert_eq!(wl.service_account, "agent-1");
    assert_eq!(wl.extra, vec!["region", "au"]);
}

#[test]
fn wimse_workload_non_matching_path_is_none() {
    let id = SpiffeId::parse("spiffe://twn.network/agent/x").unwrap();
    assert!(id.as_wimse_workload().is_none());
}

#[test]
fn wimse_workload_missing_sa_is_none() {
    let id = SpiffeId::parse("spiffe://twn.network/ns/prod/sa").unwrap();
    assert!(id.as_wimse_workload().is_none());
}

#[test]
fn wimse_workload_trust_domain_only_is_none() {
    let id = SpiffeId::parse("spiffe://twn.network").unwrap();
    assert!(id.as_wimse_workload().is_none());
}

#[test]
fn wimse_workload_wrong_keyword_is_none() {
    // `ns` present but second keyword is not `sa`.
    let id = SpiffeId::parse("spiffe://twn.network/ns/prod/xx/agent-1").unwrap();
    assert!(id.as_wimse_workload().is_none());
}

// --- SHOULD: Display / FromStr --------------------------------------------

#[test]
fn display_renders_as_uri() {
    let id = SpiffeId::parse("spiffe://twn.network/ns/prod/sa/agent-1").unwrap();
    assert_eq!(format!("{id}"), id.as_uri());
    assert_eq!(format!("{id}"), "spiffe://twn.network/ns/prod/sa/agent-1");

    let td_only = SpiffeId::parse("spiffe://twn.network").unwrap();
    assert_eq!(format!("{td_only}"), "spiffe://twn.network");
}

#[test]
fn from_str_round_trips() {
    let id: SpiffeId = "spiffe://TWN.network/x".parse().unwrap();
    assert_eq!(id.trust_domain(), "twn.network");
    assert_eq!(SpiffeId::from_str("spiffe://twn.network/x").unwrap(), id);
    assert!(SpiffeId::from_str("nope://x").is_err());
}

// --- in_trust_domain ------------------------------------------------------

#[test]
fn in_trust_domain_case_insensitive() {
    let id = SpiffeId::parse("spiffe://twn.network/x").unwrap();
    assert!(id.in_trust_domain("twn.network"));
    assert!(id.in_trust_domain("TWN.Network"));
    assert!(!id.in_trust_domain("other.network"));
    assert!(!id.in_trust_domain("twn.networ"));
}

// --- ParseError Display ---------------------------------------------------

#[test]
fn parse_error_display_is_nonempty() {
    let errs = [
        ParseError::BadScheme,
        ParseError::DisallowedComponent("port"),
        ParseError::BadTrustDomain("x"),
        ParseError::BadPath("x"),
        ParseError::TooLong,
        ParseError::Malformed,
    ];
    for e in errs {
        assert!(!format!("{e}").is_empty());
    }
}

// --- SHOULD: serde JSON round-trip ----------------------------------------

#[cfg(feature = "serde")]
mod serde_tests {
    use spiffe_id::SpiffeId;

    #[test]
    fn json_round_trip() {
        let id: SpiffeId = serde_json::from_str("\"spiffe://twn.network/x\"").unwrap();
        assert_eq!(id.trust_domain(), "twn.network");
        assert_eq!(id.path(), "x");
        let back = serde_json::to_string(&id).unwrap();
        assert_eq!(back, "\"spiffe://twn.network/x\"");
    }

    #[test]
    fn json_round_trip_canonicalises() {
        let id: SpiffeId = serde_json::from_str("\"spiffe://TWN.NETWORK/a%2Db\"").unwrap();
        let back = serde_json::to_string(&id).unwrap();
        assert_eq!(back, "\"spiffe://twn.network/a-b\"");
    }

    #[test]
    fn json_malformed_is_serde_error() {
        let r = serde_json::from_str::<SpiffeId>("\"https://twn.network/x\"");
        assert!(r.is_err());
        let r2 = serde_json::from_str::<SpiffeId>("\"spiffe://twn.network/x/\"");
        assert!(r2.is_err());
        let r3 = serde_json::from_str::<SpiffeId>("12345");
        assert!(r3.is_err());
    }
}

// --- SHOULD: property/generator test --------------------------------------

/// A tiny deterministic xorshift PRNG — keeps the test dep-free (the spec's
/// property test is a SHOULD and `proptest` is optional).
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

fn random_trust_domain(rng: &mut Rng) -> String {
    const TD: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789-._";
    let len = 1 + rng.below(20);
    let mut s = String::new();
    for _ in 0..len {
        s.push(TD[rng.below(TD.len())] as char);
    }
    s
}

fn random_segment(rng: &mut Rng) -> String {
    const SEG: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._";
    let len = 1 + rng.below(12);
    let mut s = String::new();
    for _ in 0..len {
        s.push(SEG[rng.below(SEG.len())] as char);
    }
    s
}

#[test]
fn property_random_valid_ids_round_trip() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..2000 {
        let td = random_trust_domain(&mut rng);
        let n_segs = rng.below(6); // 0..=5 segments
        let mut uri = format!("spiffe://{td}");
        for _ in 0..n_segs {
            uri.push('/');
            uri.push_str(&random_segment(&mut rng));
        }
        let id = SpiffeId::parse(&uri).unwrap_or_else(|e| panic!("rejected valid {uri}: {e:?}"));
        // as_uri is canonical and equals the input (we generate canonical
        // forms: lowercase trust domain, no percent-encoding, no trailing
        // slash, no empty segments).
        assert_eq!(id.as_uri(), uri, "as_uri mismatch for {uri}");
        let reparsed = SpiffeId::parse(&id.as_uri()).unwrap();
        assert_eq!(reparsed, id, "round-trip mismatch for {uri}");
    }
}
