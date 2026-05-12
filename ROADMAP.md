# Roadmap

`spiffe-id` is **deliberately small** — a strict parser/validator for SPIFFE / WIMSE
workload-identity *URIs*, `no_std`, zero runtime deps. Most of the roadmap is about *not* growing
the scope; the rest is hardening and a careful path to a stable `1.0`. This is a statement of intent,
not a contract — issues and PRs shape it. (Dates are omitted on purpose; this isn't release-driven.)

## Done — `0.1`

- `SpiffeId::parse` — full SPIFFE-ID validation (scheme, trust-domain & path constraints,
  percent-decoding, length caps, rejects userinfo/port/query/fragment, lowercase-canonicalises the
  trust domain); accessors (`trust_domain()`, `path()`, `as_uri()` round-trip), `Display`/`FromStr`.
- `as_wimse_workload()` — the structured `ns/<namespace>/sa/<service-account>[/<extra>...]` view,
  `None` for non-WIMSE-shaped paths (not an error).
- `ParseError` — a `match`-able set of rejection reasons.
- `#![no_std]` + `alloc`, `#![forbid(unsafe_code)]`, builds for `wasm32-unknown-unknown`; optional
  `serde` (string form); a spec-derived test suite + a doc-test.
- Project docs: this roadmap, `SECURITY.md`, the vulnerability-response & maintainer-compromise
  playbooks, the [threat model](docs/THREAT-MODEL.md), `CONTRIBUTING.md`, the implementation &
  integration guides, issue/PR templates, `CODEOWNERS`, `dependabot.yml`.

## Near term — `0.x` (likely, additive, not yet promised)

- **Fuzzing.** A `cargo-fuzz` target over `SpiffeId::parse` (round-trip: parse → `as_uri()` → parse
  is idempotent; never panics; never produces a `SpiffeId` whose `as_uri()` doesn't re-parse),
  wired into CI on a schedule. *(This is the highest-priority hardening item.)*
- **`cargo audit` in CI.** Even at zero deps — so the crate appearing in RustSec, or a future
  dep/dev-dep advisory, is caught.
- **More accessors, if real use justifies them** — e.g. `is_member_of(&trust_domain)`, a path-segment
  iterator, `last_segment()`, `parent()`/`child()` (append/strip a path segment), a `TrustDomain`
  newtype so trust domains can be parsed/validated/compared on their own and SPIFFE IDs built from
  them. These are *candidates* — each lands only if it pulls weight (a consumer needs it, it doesn't
  bloat the API). Some of this exists as a prototype; folding it in is a `0.x` minor at most.
- **`PartialEq<str>` / `AsRef<str>` ergonomics** and other small "make it pleasant to use" trait
  impls, where they don't risk a footgun (e.g. accidentally comparing the un-canonicalised string).
- **Property tests** in the normal test suite (in addition to fuzzing) for the round-trip and
  canonicalisation invariants.
- **No-std target check in CI on a bare-metal target** (`thumbv*-none-eabi`) in addition to wasm32,
  to keep "really `no_std`" honest.

## `1.0` — when the API has earned stability

`1.0` ships when: the parsing surface has been stable through real use for a while, the fuzz target
has run without finding anything new, the public API is one we're happy to commit to under semver
for the long haul, and the docs (esp. the threat model) have survived contact with consumers. `1.0`
is a *stability* commitment, not a feature gate — it might have *fewer* knobs than `0.x`, not more.
After `1.0`: semver as you'd expect; security backports to the previous major for 12 months.

## Explicitly out of scope — won't be in this crate

These are real and useful, just **not here** — a PR adding them will be declined (with thanks):

- **SVID documents.** X.509-SVID (the certificate, its chain, its validity), JWT-SVID (the token,
  its signature, `exp`/`aud`/`iss`). `spiffe-id` parses the *ID URI* those carry; verifying the
  documents is a different concern. (See the [`spiffe`](https://crates.io/crates/spiffe) crate.)
- **Trust bundles & federation.** Trust-domain → key-set mappings, the SPIFFE trust-bundle format,
  federation policy between trust domains.
- **The SPIFFE Workload API.** Fetching SVIDs from a SPIRE-style agent (the gRPC API, the
  socket-discovery dance, rotation handling).
- **The WIMSE Workload Identity Token (WIT)** itself, or the WIMSE proof-of-possession / transaction
  token machinery. `spiffe-id` understands the *identifier* a WIT carries; it isn't a WIT
  implementation.
- **IDNA / Unicode trust domains.** SPIFFE trust domains are an ASCII subset by spec; we won't add
  Unicode normalisation/punycode handling (and we think the spec is right not to — it sidesteps a
  whole class of confusable-domain problems).
- **Anything I/O, async, time-, or crypto-dependent.** The crate is a pure function from bytes to a
  parse result; it's going to stay that way. That's what makes it usable in `wasm32`/embedded and
  trivially testable.

## How to influence this

Open an issue or a [Discussion](https://github.com/sanguinehost/wimse/discussions). "I'm using this
crate and I need X" or "the spec says Y and the parser does Z" are the most useful kinds of input.
For anything that would change the *scope* (the lists above), expect a conversation before a PR.
