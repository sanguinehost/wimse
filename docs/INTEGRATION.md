# Integration & deployment guide — `spiffe-id`

`spiffe-id` is a library, so "deployment" means: how you take a dependency on it, how you wire it
into your service / wasm app / embedded firmware, what version-and-feature choices to make, and the
supply-chain story. For the API itself see [`FEATURES.md`](../FEATURES.md) / the
[rustdoc](https://docs.rs/spiffe-id); for the trust-boundary do's and don'ts (the part that bites
people), see [`docs/THREAT-MODEL.md`](THREAT-MODEL.md) — read it before you ship.

## Add the dependency

```toml
# Cargo.toml
[dependencies]
spiffe-id = "0.1"                                  # the base crate: no_std + alloc, zero runtime deps
# or, with serde (de)serialisation:
spiffe-id = { version = "0.1", features = ["serde"] }
```

There's one feature, `serde`. The crate is `no_std` + `alloc` in every configuration — there's
nothing extra to set for a wasm32 / embedded consumer, and there is no `std` feature (it impls
`core::error::Error` unconditionally). **MSRV:** Rust 1.81 (declared in the crate's `rust-version`).

### Version pinning

- **Pre-1.0:** a `0.x` minor bump can be breaking (that's how `0.x` semver works); pin
  `spiffe-id = "0.1"` (= `^0.1`, i.e. `>=0.1.0, <0.2.0`) and read [`CHANGELOG.md`](../CHANGELOG.md)
  before you bump the minor. Security fixes ship as patches on the current minor (and best-effort
  backports to the previous minor for ~90 days — see [`SECURITY.md`](../SECURITY.md)).
- **If you re-export `spiffe-id` types in *your* public API:** your users now feel its breaking
  changes — be conservative about bumping, and document the dependency. Your effective MSRV is
  `max(yours, 1.81)`.
- **Lockfiles:** commit your `Cargo.lock` for binaries/services so a rebuild is reproducible; don't
  for libraries. (A security fix that needs `cargo update -p spiffe-id` is then a one-liner.)

## Wiring it into a service

The pattern (and the threat-model rules say this loudly): **parse at the boundary, then carry the
typed value; don't carry strings.**

```rust
use spiffe_id::SpiffeId;

// 1. AUTHENTICATE first. If the ID arrives in a JWT, you've already verified the signature,
//    `iss`, `aud`, `exp`. If in an X.509-SVID, you've validated the chain against the right
//    trust bundle. `spiffe-id` parses the *string*; it does not make it trustworthy.
let raw_sub: &str = /* the `sub` claim of a token you've verified */;

// 2. PARSE — turn the (now-authenticated) string into a typed SpiffeId, or reject.
let id: SpiffeId = raw_sub.parse().map_err(|e| reject("bad SPIFFE ID", e))?;

// 3. AUTHORIZE on the structured, authenticated identity — with a trust-domain ALLOWLIST,
//    exact-match on the components. Never substring/regex the URI string.
if !TRUSTED_DOMAINS.contains(id.trust_domain()) {
    return Err(reject("unknown trust domain", id.trust_domain()));
}
if let Some(wl) = id.as_wimse_workload() {
    enforce_policy(wl.namespace, wl.service_account)?;          // your authz
} else {
    // a valid SPIFFE ID with a non-WIMSE-shaped path — YOUR call what to do (allow? deny? log?).
    handle_non_wimse_id(&id)?;
}

// 4. Carry `id` (the typed value) through the request, not `raw_sub`. Compare SpiffeId values,
//    not strings. Re-encode for the destination grammar (SQL, logs, URLs, a SpiceDB object id)
//    whenever you put it somewhere new.
```

Things to get right (the full list is in [`docs/THREAT-MODEL.md`](THREAT-MODEL.md#audience-1--you-operate-a-service-that-parses-spiffe-ids-from-untrusted-input)):

- **Bound the input before you parse it.** The crate caps at the SPIFFE-spec limits (2048 bytes
  total / 255 for the trust domain); your transport (HTTP body limits, JWT size limits, config
  value limits) bounds it earlier. Don't hand the parser a megabyte string.
- **A valid SPIFFE ID is not authn and not authz.** The trust domain is whatever was in the input,
  not a verified issuer. Gate on the *verified* identity; allowlist trust domains.
- **`as_wimse_workload() == None`** means "this path isn't WIMSE-workload-shaped" — not an error,
  not a deny. Decide your policy.
- **Compare `SpiffeId`s, not strings** (canonicalisation makes `SPIFFE://EXAMPLE.org/%61` and
  `spiffe://example.org/a` equal); use `==` / `in_trust_domain()`.
- **Re-encode for the destination.** A clean canonical SPIFFE URI is still attacker-influenced text;
  escape it for SQL / logs / URLs / wherever it lands.

## Wiring it into a wasm32 app or embedded firmware

The crate is `#![no_std]` + `alloc` with no I/O, no clock, no randomness — so it drops into a
browser (`wasm32-unknown-unknown`, e.g. inside a WASM-compiled identity component) or onto a
microcontroller that has a global allocator. Nothing special to configure; just depend on it. The
CI builds it for `wasm32-unknown-unknown` on every push. (For a bare-metal `thumbv*-none-eabi`
target you provide the `#[global_allocator]` in your binary, as usual; the library itself doesn't
need one.)

## Where it fits in a SPIFFE/WIMSE pipeline

`spiffe-id` is the **identifier layer**. A fuller stack around it:

```
  ┌─ obtain an SVID ────────────────────────────────────────────┐
  │  • from a SPIRE-style agent (the SPIFFE Workload API)        │  ← not this crate
  │  • or from a JWT-SVID / X.509-SVID handed to you            │
  └──────────────────────────────────────┬──────────────────────┘
                                          │
  ┌─ verify the SVID document ────────────▼──────────────────────┐
  │  • JWT-SVID: signature, iss, aud, exp                        │  ← not this crate
  │  • X.509-SVID: chain against the trust bundle for the domain │     (see the `spiffe` crate, or
  └──────────────────────────────────────┬──────────────────────┘      jsonwebtoken / x509-parser / …)
                                          │  the SVID's subject = a SPIFFE ID *string*
  ┌─ parse & validate the SPIFFE ID ──────▼──────────────────────┐
  │  SpiffeId::parse(&sub)?  →  trust_domain(), path(),          │  ← THIS CRATE
  │  as_wimse_workload(), in_trust_domain(), as_uri()            │
  └──────────────────────────────────────┬──────────────────────┘
                                          │  a typed, canonical, authenticated identity
  ┌─ authorize ───────────────────────────▼──────────────────────┐
  │  trust-domain allowlist; policy on the structured components │  ← your code
  └─────────────────────────────────────────────────────────────┘
```

If you need the parts marked "not this crate" (the Workload API client, SVID-document handling,
trust bundles, federation), the [`spiffe`](https://crates.io/crates/spiffe) crate has the full
stack; `spiffe-id` deliberately stays the slim, dependency-free, `no_std` ID layer those build on.

## Supply chain

- **Zero runtime dependencies; no build script.** Nothing transitive to vet, nothing executes at
  build time. `cargo vet` / `cargo deny` see one crate (plus `serde`/`serde_json` if you enable
  `serde` / for dev). `cargo audit` will flag this crate if it ever appears in [RustSec](https://rustsec.org/)
  — which is exactly what you want, and another reason the crate is zero-dep (the blast radius is
  just the crate itself).
- **Verify what you pin.** The canonical name is `spiffe-id` on crates.io; the canonical repo is
  `https://github.com/sanguinehost/wimse`. A published version's contents match this repo at the
  corresponding `vX.Y.Z` tag (you can diff a downloaded `.crate`). A *yanked* version is one we've
  pulled — `cargo update` will move you off it; pay attention to `cargo audit`/RustSec for *why*.
- **Reproducibility.** Pin via your `Cargo.lock`; the crate has no env-dependent or codegen-time
  behaviour, so the same source builds the same library.
- **If you suspect a problem with the published crate** (a version that doesn't match the repo, an
  unexpected owner, a backdoored release), that's a security report — `security@twn.systems` (see
  [`SECURITY.md`](../SECURITY.md)), and the [maintainer-compromise playbook](playbooks/maintainer-compromise.md)
  is what we'd run.

## Common questions

- **"Should I use this or the `spiffe` crate?"** — Use `spiffe-id` if all you need is "is this
  string a valid SPIFFE ID, and what are its parts" (parsing a `sub` claim, a URI SAN, a config
  value), especially in a `no_std`/wasm/embedded context or where you want zero deps. Use `spiffe`
  if you need the Workload API client, SVID-document handling, or trust bundles — that crate is
  heavier and `std`.
- **"Can a `SpiffeId` be invalid?"** — No. If you have one, it passed every spec check at parse
  time, and its accessors return the canonical components. You don't need to re-validate it.
- **"Why is the path percent-decoded?"** — So comparison is on the *meaning*, not the encoding;
  `…/%61` and `…/a` are the same path. Re-encode if you put it into a URL/grammar that needs it.
- **"`as_wimse_workload()` returns `None` for my ID — bug?"** — No. It only matches the
  `ns/<ns>/sa/<sa>[/extra]` shape; everything else (a bare path, a different convention, a
  trust-domain-only ID) is `None`. The ID is still valid; it's just not a WIMSE *workload* ID.
- **"It rejected `spiffe://td/`!"** — Correct — the SPIFFE spec forbids a trailing slash; the
  trust-domain-only ID is `spiffe://td` (no slash). (We reject what `go-spiffe` rejects; see the
  [implementation guide](IMPLEMENTATION.md).)
