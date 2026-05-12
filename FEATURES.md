# Features & capabilities — `spiffe-id`

What this crate does, what it gives you, and (just as important) what it deliberately doesn't. For
the *why* and the future, see [`ROADMAP.md`](ROADMAP.md); for the trust-boundary story, the
[threat model](docs/THREAT-MODEL.md); for how to use it, the
[integration guide](docs/INTEGRATION.md); for how it works inside, the
[implementation guide](docs/IMPLEMENTATION.md). The authoritative API reference is the rustdoc on
[docs.rs](https://docs.rs/spiffe-id) — this page is the orientation.

## In one line

A strict, `no_std`, zero-runtime-dependency parser/validator for **SPIFFE-ID URIs**
(`spiffe://<trust-domain>/<path>`) and the **WIMSE workload-identifier** path grammar
(`ns/<namespace>/sa/<service-account>[/<extra>...]`), with a typed result you can pull components
out of and round-trip back to the canonical URI.

## What it parses & validates

`SpiffeId::parse(&str) -> Result<SpiffeId, ParseError>` (also `str::parse` / `FromStr`) enforces the
[SPIFFE-ID spec](https://github.com/spiffe/spiffe/blob/main/standards/SPIFFE-ID.md):

| Rule | Behaviour |
|---|---|
| Scheme | `spiffe` — accepted **case-insensitively** on input (`SPIFFE://…` works), lowercased in the canonical form; any other scheme → `ParseError::BadScheme`. The `//` authority marker is mandatory. |
| Authority = the trust domain | non-empty, ≤ 255 bytes, only the spec-allowed characters; **no** userinfo (`user@…`), **no** port (`…:1234`) → `ParseError::BadTrustDomain` / `ParseError::DisallowedComponent`. The trust domain is **lowercase-canonicalised** so `EXAMPLE.org` and `example.org` are the same. |
| Path | `/`-separated segments; **no** empty segments (so no `//`, no trailing `/`); **no** `.` or `..` segments; only the spec-allowed characters; **percent-encoding** is decoded and must be well-formed → `ParseError::BadPath`. A SPIFFE ID may have an empty path (`spiffe://td` — a trust-domain-only ID, valid per the spec). |
| Query / fragment | not allowed → `ParseError::DisallowedComponent`. |
| Total length | ≤ 2048 bytes → `ParseError::TooLong`. |
| Canonical form | `as_uri()` returns the validated, canonicalised URI (lowercased scheme + trust domain, percent-decoded path, no trailing slash); `parse(x).unwrap().as_uri()` re-parses to an equal value (round-trip). |

`SpiffeId` implements `Display` (the canonical URI), `FromStr`, `Clone`, `PartialEq`/`Eq`/`Hash`
(over the canonical form — compare *these*, not raw strings), and `Debug`.

## Accessors

- `trust_domain() -> &str` — the (lowercased) trust domain.
- `path() -> &str` — the path, percent-decoded, without a leading slash (`agent/orchestrator`), `""`
  for a trust-domain-only ID.
- `path_segments() -> Vec<&str>` — the path split on `/`; empty `Vec` for a trust-domain-only ID.
- `in_trust_domain(td: &str) -> bool` — does this ID belong to that trust domain? (exact,
  case-insensitive comparison).
- `as_uri() -> String` / `Display` — the canonical `spiffe://…` URI.
- `as_wimse_workload() -> Option<WimseWorkloadId<'_>>` — if the path matches the WIMSE workload
  grammar `ns/<namespace>/sa/<service-account>[/<extra>...]` (with `<namespace>` and
  `<service-account>` non-empty), a (borrowed) struct with `.namespace`, `.service_account`, and
  `.extra` (the remaining segments). **`None` means "this path isn't WIMSE-workload-shaped"** — it
  is *not* an error (never errors) and *not* a security verdict; deciding what your service does with
  non-WIMSE paths is your call (see the threat model).
- `FromStr` (`"…".parse::<SpiffeId>()`), `Clone`, `Debug`, `PartialEq`/`Eq`/`Hash` (over the
  canonical form — compare *these*, not raw strings).

(A few more ergonomic accessors — a `TrustDomain` newtype, a segment *iterator* alongside the `Vec`,
`parent`/`child` for appending/stripping a path segment — are on the [roadmap](ROADMAP.md) as `0.x`
additive candidates; check the rustdoc for what's actually shipped in your version.)

## The error type

`ParseError` is a small, `match`-able enum of rejection reasons — `BadScheme`,
`DisallowedComponent(&'static str)` (userinfo / port / query / fragment),
`BadTrustDomain(&'static str)`, `BadPath(&'static str)`, `TooLong`, `Malformed`. It implements
`Display` (a human-readable message), `Debug`, `Clone`, `PartialEq`/`Eq`, and
`core::error::Error` — i.e. `std::error::Error` *without needing `std`* (stable since Rust 1.81), so
it slots into `anyhow`/`?` chains in both `std` and `no_std` consumers. Match on it when you want to
react to *which* rule failed (the `&'static str` payloads narrow it further); `Display` it when you
just want to log the reason.

## Cargo features

The crate has **one** feature. It's `no_std` + `alloc`, zero runtime deps, `#![forbid(unsafe_code)]`
in *every* configuration — there is no `std` feature (the `core::error::Error` impl is
unconditional).

| feature | default | what it does |
|---|---|---|
| `serde` | off | `Serialize` / `Deserialize` for `SpiffeId` — serialises as the canonical URI string, deserialises by *parsing* it (so a `SpiffeId` field in a struct can never hold an invalid value, even off the wire). Pulls in `serde` (`default-features = false`, `alloc` + `derive`); dev-tested with `serde_json`. |

```toml
# server / general use:
spiffe-id = "0.1"

# with serde:
spiffe-id = { version = "0.1", features = ["serde"] }

# (the crate is already no_std + alloc — nothing extra to set for a wasm32 / embedded consumer)
```

## Properties (the "non-functional" features)

- **`#![no_std]` + `alloc` only.** No `std`, no I/O, no clock, no randomness, no global state, no
  build script. Builds for `wasm32-unknown-unknown` (and bare-metal targets that provide an
  allocator).
- **`#![forbid(unsafe_code)]`.** Memory-safety isn't something you have to take on trust here.
- **Zero runtime dependencies.** `serde` is opt-in; `serde_json` is a dev-dependency only. Trivially
  `cargo vet` / `cargo deny` friendly; nothing executes at build time.
- **Deterministic & total-ish.** Parsing is a pure function: same input → same `Result`, every time;
  it doesn't panic on any input (over-long inputs are rejected, not crashed on). Allocations are
  bounded by the spec's 2048-/255-byte caps.
- **Strict.** Rejects what the spec says to reject (and what `go-spiffe` rejects) — the trailing
  slash, the relative-path segment, the embedded port, bad percent-encoding — rather than silently
  accepting it to bite a consumer later.
- **Spec-tested.** The test suite is built from the SPIFFE-ID spec's rules and rejection cases; it's
  the compliance evidence. A round-trip property (`parse → as_uri → parse` is idempotent) is
  checked, with a fuzz target [planned](ROADMAP.md).

## What it is *not*

It's a **syntactic gate**, not security. A parsed `SpiffeId` says the input *looks like* a SPIFFE ID
and what its parts are — **not** that it's authenticated, authorised, or issued by anyone you trust.
The trust domain is whatever string was in the input, not a verified issuer. Authn (the JWT
signature, the X.509 chain, the Workload-API attestation) and authz happen in *your* code, on inputs
this crate has shaped. The [threat model](docs/THREAT-MODEL.md) is blunt about this.

And it doesn't do SVID documents (X.509-SVID / JWT-SVID), trust bundles, federation, or the SPIFFE
Workload API — see [`ROADMAP.md`](ROADMAP.md) for why those live elsewhere (the
[`spiffe`](https://crates.io/crates/spiffe) crate has the full stack; `spiffe-id` is the slim ID
layer it builds on).
