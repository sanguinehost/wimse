# Implementation guide — `spiffe-id`

How the crate works inside: the parsing algorithm, the data layout, the `no_std` strategy, the test
strategy, and how to extend it without breaking the invariants. For *what* it does, see
[`FEATURES.md`](../FEATURES.md); for *why* the scope is what it is, [`ROADMAP.md`](../ROADMAP.md);
for the trust-boundary story, [`docs/THREAT-MODEL.md`](THREAT-MODEL.md). The authoritative reference
is the rustdoc; this is the map.

## Specs it implements

- **SPIFFE-ID** — <https://github.com/spiffe/spiffe/blob/main/standards/SPIFFE-ID.md> (the URI
  format: scheme, trust-domain authority, path; the character sets; the length caps; the
  disallowed components).
- **WIMSE** — <https://datatracker.ietf.org/wg/wimse/about/> — for the workload-identifier path
  convention `ns/<namespace>/sa/<service-account>[/<extra>...]` that `as_wimse_workload()` recognises.
- The exact behaviour list (the "Behavior N" comments in `src/lib.rs`) and the test plan come from
  `docs/specs/iter-18-spiffe-id.md` in [`sanguinehost/ferrousgate`](https://github.com/sanguinehost/ferrousgate)
  — that's the design doc; this crate is its implementation. If you're touching the parser, read it.

## Layout

It's a single-module crate: `src/lib.rs` (the implementation + unit tests) and `tests/spiffe_id.rs`
(the spec-plan integration tests). No build script, no proc-macros, no submodules — small on
purpose.

```
src/lib.rs            ── the crate: SpiffeId, ParseError, WimseWorkloadId, the parse pipeline,
                         the validators, the serde impls (#[cfg(feature = "serde")]), #[cfg(test)] units
tests/spiffe_id.rs    ── the SPIFFE-ID-spec "Test plan" as #[test]s (the compliance evidence)
Cargo.toml            ── package metadata, the `serde` feature, the `[lints]` table, MSRV
.github/workflows/ci.yml  ── the gate: check / no-default-features / wasm32 / serde test / clippy / fmt
```

## The data type

```rust
pub struct SpiffeId {
    trust_domain: String,   // canonical: lowercase, no `spiffe://`, no userinfo, no port
    path: String,           // canonical: percent-decoded, NO leading slash, no trailing slash; "" for a trust-domain-only ID
}
```

`SpiffeId` holds the **canonical** decomposition, not the original string. So:

- `parse(s)` validates *and canonicalises* — it lowercases the scheme/trust-domain and
  percent-decodes the path; the original casing/encoding of `s` is gone.
- `as_uri()` rebuilds the canonical URI from the parts: `"spiffe://"` + `trust_domain` + (`"/"` +
  `path` if the path is non-empty). It allocates a fresh `String`.
- **Round-trip:** `SpiffeId::parse(&id.as_uri()) == Ok(id.clone())` for any valid `id` — `as_uri()`
  emits a form that re-parses to the same value. (This is a test invariant; a fuzz target for it is
  on the roadmap.)
- `Eq`/`Hash` are derived over `(trust_domain, path)` — i.e. over the **canonical** form. Two
  different input strings that canonicalise the same (`SPIFFE://EXAMPLE.org/x` and
  `spiffe://example.org/x`, or `…/%61` and `…/a`) compare equal. **Always compare `SpiffeId`s, not
  raw strings.**

`ParseError` is a plain enum (`#[derive(Debug, Clone, PartialEq, Eq)]`) with `&'static str`
payloads on the "which sub-rule" variants — cheap, `match`-able, `no_std`-friendly. It implements
`Display` and `core::error::Error` (so it works in `?`/`anyhow` chains under `std` *and* `no_std`).

`WimseWorkloadId<'a>` borrows from the `SpiffeId`'s `path` — `namespace: &'a str`,
`service_account: &'a str`, `extra: Vec<&'a str>` (the segments after `sa/<service-account>`). It's
a *view*, not an owned value; produced only by `as_wimse_workload()`, which never errors (returns
`None` for a path that isn't `ns/_/sa/_[…]`-shaped).

## The parse pipeline (`SpiffeId::parse`)

In order — each step either consumes a slice and passes the rest along, or bails with a specific
`ParseError`. No allocation happens until we know the input is well-formed (the canonical `String`s
are built from validated slices). No `unsafe`, ever.

1. **Length cap.** `s.len() > 2048` → `ParseError::TooLong`. (Bound *before* doing any work; this
   is the algorithmic-DoS floor — everything after is linear in a now-bounded length.)
2. **Scheme.** `split_scheme(s)` — strip a leading `spiffe` *case-insensitively* and the `:`,
   returning the rest; not `spiffe…` → `ParseError::BadScheme`. (The spec's scheme is `spiffe`;
   URI schemes are case-insensitive, so input case is accepted but the canonical form is `spiffe`.)
3. **Authority marker.** Strip the mandatory `//` (a SPIFFE ID is always `spiffe://authority…`);
   absent → `ParseError::Malformed`.
4. **Split off the authority.** The authority runs up to the first `/`, `?`, or `#`
   (`split_at_any`); the rest is the "tail".
5. **Reject userinfo / port.** `@` in the authority → `DisallowedComponent("userinfo")`; `:` in the
   authority → `DisallowedComponent("port")` (a SPIFFE authority has no userinfo, so a `:` can only
   be a port separator — and an IPv6-literal host with `:` isn't a valid trust domain anyway).
6. **Validate the trust domain** (`validate_trust_domain`): non-empty; ≤ 255 bytes; only the
   spec-allowed characters; then **lowercase it** into the canonical `trust_domain`. Failures →
   `BadTrustDomain(why)`.
7. **The tail** is one of three things:
   - **empty** → trust-domain-only ID; `path = ""`.
   - **`/` + the rest** → strip the `/`, split the path body off any trailing `?…`/`#…`
     (`split_at_any`); if there *was* a query/fragment → `DisallowedComponent`; otherwise
     `validate_path(path_body)` (see below) → the canonical `path`. (`spiffe://td/` lands here with
     an *empty* `path_body`, which `validate_path` rejects — that's how the "no trailing slash" rule
     is enforced, along with `spiffe://td/a/` rejecting because its last segment is empty.)
   - **`?…` or `#…`** (the tail started with one) → `DisallowedComponent`.
8. Done — return `SpiffeId { trust_domain, path }`.

### `validate_path`

Given the path *body* (no leading slash, may be `""`):

- An empty body (which is what `spiffe://td/` produces, after stripping the `/`) → `BadPath` —
  there's no valid "empty path with a slash".
- Split on `/` into segments. **Every** segment must be: non-empty (so `//` → `BadPath`, and a
  trailing `/` makes the last segment empty → `BadPath`); not `.` and not `..` (the relative-path
  segments are forbidden); over the spec-allowed character set; and if it contains `%`, a
  well-formed percent-encoding (`%HH` with hex digits) — which is **decoded** into the canonical
  form. Bad anything → `BadPath(why)`.
- The reassembled, percent-decoded path (joined with `/`) is the canonical `path`.

### `as_wimse_workload`

Pure function on the parsed `path`: split into segments; if `segments == ["ns", <ns>, "sa", <sa>,
<extra>…]` with `<ns>` and `<sa>` non-empty, return `Some(WimseWorkloadId { namespace: <ns>,
service_account: <sa>, extra: <the rest> })` (all borrowed from `path`); else `None`. **Never
errors** — a non-WIMSE-shaped path is not a parse failure, the ID is still valid, it just isn't a
WIMSE workload ID.

## `no_std` strategy

- `#![no_std]` at the crate root; `extern crate alloc;`. The crate uses `core::*` and
  `alloc::{string::String, vec::Vec}` — nothing from `std`. There is **no `std` feature** and
  nothing changes at runtime by configuration except `serde`.
- `ParseError` impls `core::error::Error` directly (stable since Rust 1.81 — that's the MSRV
  driver). So it works in `?`/`anyhow` chains for `std` *and* `no_std` consumers without a feature
  flag.
- No I/O, no clock, no randomness, no global/static mutable state, no `thread_local!`. `parse` is a
  pure function `(&str) -> Result<SpiffeId, ParseError>`.
- Result: it builds for `wasm32-unknown-unknown` (in CI) and for any target that provides a global
  allocator. (A bare-metal `thumbv*-none-eabi` build of the *library* works too — the binary that
  links it provides the allocator.)
- Allocations are minimal and bounded: one `String` for the trust domain, one for the path, small
  `Vec<&str>`s for `path_segments()`/`as_wimse_workload()` — all capped by the 2048/255-byte input
  caps. No intermediate buffers; the canonical `String`s are built from validated input slices.

## `serde` (feature `serde`)

Behind `#[cfg(feature = "serde")]`: `SpiffeId` serialises as its canonical URI string (`as_uri()`)
and deserialises by **parsing** that string with `SpiffeId::parse` — so a `SpiffeId` field on a
struct (a JWT claim set, a config) can never hold an invalid value, even when it came off the wire;
a bad string is a deserialisation *error*, not a malformed `SpiffeId`. The feature pulls in `serde`
with `default-features = false` + `alloc` + `derive`; `serde_json` is a dev-dependency for the
round-trip tests. (`WimseWorkloadId` borrows, so it isn't `Deserialize`; if you need to persist it,
persist the `SpiffeId` and call `as_wimse_workload()` on the way back out.)

## Test strategy

- **`tests/spiffe_id.rs`** mirrors the "Test plan" section of the iter-18 spec — one `#[test]` per
  MUST/MUST-NOT vector: valid IDs with/without paths, the canonicalisation cases (uppercase scheme,
  uppercase trust domain), every rejection (bad scheme, userinfo, port, query, fragment, empty
  segment, trailing slash, `.`/`..`, bad percent-encoding, over-length, missing trust domain),
  `as_uri()` round-trip, `path_segments()`, `in_trust_domain()`, and the `as_wimse_workload()`
  cases (matching and non-matching paths). This file is the **compliance evidence** — if you found a
  spec corner the parser gets wrong, the best bug report is a failing test added here.
- **`#[cfg(test)]` units in `src/lib.rs`** cover the internal helpers (`split_scheme`,
  `split_at_any`, `validate_trust_domain`, `validate_path`, the percent-decode, `eq_ignore_ascii_case`)
  in isolation, plus property-ish round-trip checks.
- **CI** (`.github/workflows/ci.yml`) runs: `cargo check` (default), `cargo check --no-default-features`,
  `cargo check --target wasm32-unknown-unknown`, `cargo check --features serde`,
  `cargo test --features serde`, `cargo clippy --all-targets --features serde -- -D warnings`, and
  `cargo fmt --check` — all with `RUSTFLAGS: -D warnings`.
- **Planned:** a `cargo-fuzz` target over `SpiffeId::parse` (never panics; `parse → as_uri → parse`
  is idempotent; a produced `SpiffeId`'s `as_uri()` always re-parses) on a CI schedule — top of the
  [roadmap](../ROADMAP.md) hardening list.

## Extending it without breaking things

The invariants you must not violate when changing the parser/types:

1. **`#![forbid(unsafe_code)]`.** No exceptions.
2. **`no_std` + `alloc` only.** No `std`. New allocations from `alloc::*`; new `format!` is
   `alloc::format!`. (CI's `--no-default-features` and `wasm32` checks enforce this.)
3. **Zero new runtime deps.** `serde` (optional) and `serde_json` (dev) are the whole dependency
   list, and we'd like to keep it that way.
4. **Canonical-form invariant.** A `SpiffeId` is *always* canonical (lowercase scheme/trust-domain,
   percent-decoded path, no trailing slash). Any new constructor must canonicalise; any new accessor
   must return the canonical thing. `parse → as_uri → parse` idempotence must hold.
5. **Strictness is a feature.** Don't loosen validation "to be helpful". If the SPIFFE spec says
   reject it, reject it. If you want to *add* a stricter check the spec implies, that's a behaviour
   change → semver-relevant → note it.
6. **Scope.** New accessors on `SpiffeId` (a `TrustDomain` newtype, `parent`/`child`, a segment
   iterator) are fair game — small, additive, justified by real use (a candidate list is in the
   roadmap). New *responsibilities* (SVID documents, trust bundles, the Workload API, async, I/O,
   crypto, time) are out — see the roadmap's "won't be in this crate" list.
7. **Tests track the spec.** Any parser change comes with the test(s) for the new/changed behaviour
   in `tests/spiffe_id.rs` (and the units in `src/lib.rs` for any new helper).

When in doubt about a parsing decision, the order of authority is: the SPIFFE-ID spec → the iter-18
spec doc in ferrousgate → what `go-spiffe` does → ask in an issue.
