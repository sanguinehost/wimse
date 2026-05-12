# Threat model — `spiffe-id`

This document is honest about what `spiffe-id` defends against, what it deliberately does not, and
what is therefore on *you*. It's written for three audiences — read the section that's you (or all
of them). It is not a substitute for threat-modelling *your* system; it's the part of that exercise
that's about this dependency.

The high-level mental model: **`spiffe-id` is a strict syntactic gate.** It turns a byte string into
either a typed, well-formed `SpiffeId` (and an optional WIMSE workload view) or a precise
`ParseError`. That's the whole job. A parsed `SpiffeId` tells you the input *looks like* a SPIFFE
ID and what its parts are — nothing about whether it's *true*, *authorised*, or *issued by anyone
you trust*. Authentication and authorization happen elsewhere, on inputs this crate has shaped.

---

## What the crate is, mechanically

- **Inputs:** a `&str` (or `String`) from anywhere — config, a header, a JWT `sub`, an X.509 SAN,
  a database row, an attacker's request body.
- **Outputs:** `Result<SpiffeId, ParseError>` (and, on a parsed `SpiffeId`, accessors:
  `trust_domain()`, `path()`, `as_uri()`, `as_wimse_workload()`, …).
- **Properties:** `#![no_std]` + `alloc` only; `#![forbid(unsafe_code)]`; zero runtime dependencies
  (optional `serde` for string (de)serialisation); no I/O, no clocks, no randomness, no global
  state; deterministic; allocates an owned `String` for the canonical form and small `Vec`s for
  path segments — bounded by the SPIFFE-spec caps (2048 bytes total, 255 for the trust domain).
- **What "valid" means here:** scheme `spiffe` (accepted case-insensitively on input, lowercased in
  the canonical form), with the mandatory `//` authority marker; an authority that is a non-empty,
  ≤255-byte trust domain over the allowed character set, lowercase-canonicalised, with no userinfo
  and no port; a path that is `/`-separated, non-empty, non-`.`/`..` segments over the allowed
  character set with valid percent-encoding and no trailing slash (the path may be empty —
  `spiffe://td` is a valid trust-domain-only ID); no query and no fragment; total length ≤ 2048
  bytes. (See [`docs/IMPLEMENTATION.md`](IMPLEMENTATION.md) for the exact algorithm and the spec
  references.)

### Assets & trust boundaries

| Asset | Threat if it fails | Who owns the boundary |
|---|---|---|
| Correctness of the accept/reject decision | A should-reject ID gets through (→ downstream confusion) or a should-accept ID is rejected (→ availability) | this crate |
| Correctness of the parsed components | `trust_domain()` / `path()` / the WIMSE workload view returns the wrong thing (→ a downstream check keys off the wrong value) | this crate |
| Bounded resource use on a bounded input | super-linear CPU/memory in the parser → algorithmic DoS | this crate (parser); **you** (bounding the input size before you call it) |
| "This ID belongs to whom it claims" | impersonation | **not this crate** — your authn (the JWT signature, the X.509 chain, the Workload API attestation) |
| "This ID is allowed to do X" | privilege escalation | **not this crate** — your authz |
| Integrity of the published crate | a poisoned `spiffe-id` on crates.io | this crate's maintainers + the supply chain (see the playbooks) |

The big line: **the crate's boundary is *syntax*. Everything about *meaning* is yours.**

---

## Audience 1 — you operate a service that parses SPIFFE IDs from untrusted input

(An authz proxy, a gateway, an API server, a workload that consumes a JWT-SVID's `sub` or an
X.509-SVID's URI SAN, etc. This is the audience that matters most.)

### Threats this crate handles for you

- **Malformed / hostile syntax.** Wrong scheme, missing/empty trust domain, an authority carrying
  userinfo or a port, a query or fragment, empty path segments (`//`, trailing `/`), `.`/`..`
  segments, bad percent-encoding, over-length input — all rejected with a specific `ParseError`.
  You get a clean "no" instead of an attacker-shaped string flowing further in.
- **Length-bomb in the *parser*.** The 2048-byte (total) and 255-byte (trust domain) caps are
  enforced; the parsing work is linear in the (now-bounded) input. The crate won't blow up CPU or
  memory on a bounded input.
- **Casing / normalisation surprises.** The scheme and the trust domain are lowercase-canonicalised,
  so `SPIFFE://EXAMPLE.org/...` and `spiffe://example.org/...` parse to the same `SpiffeId` (same
  trust domain, same `as_uri()`) — you don't have to remember to lowercase before comparing, and
  `in_trust_domain()` / `PartialEq` already do the case-insensitive thing. (The *path* is **not**
  case-folded — `…/Agent` and `…/agent` are different paths, per the spec; only the trust domain
  and scheme are.)
- **A "valid SPIFFE ID" that isn't.** Strictness means `go-spiffe`-style edge cases (the trailing
  slash, the relative-path segment, the embedded port) are rejected here, not silently accepted to
  bite you later.

### Threats this crate does **not** handle — they're yours

1. **A parsed `SpiffeId` is not authenticated.** If it came out of a JWT, *you* must have verified
   the JWT's signature, issuer, audience, and expiry — `spiffe-id` parses the `sub` string, it
   doesn't know the token was real. If it came out of an X.509 SVID, *you* must have validated the
   certificate chain against the right trust bundle. If you accept a SPIFFE ID a client simply
   *asserts* (in a header, a query param) with no proof, you've built impersonation-by-design.
2. **A parsed `SpiffeId` is not authorised.** `trust_domain() == "example.org"` does not mean "this
   request may do X". Do your authz on the *verified* identity, not on the *parsed string*. Don't
   substring-match the URI for authz decisions — match on the structured components
   (`trust_domain()`, the path / `as_wimse_workload()`), exact, after authentication.
3. **The trust domain is whatever was in the input.** It is *not* a verified issuer. Two different
   issuers can both put `spiffe://example.org/...` in a token; only the *signature* (and the trust
   bundle you tie that trust domain to) makes it trustworthy. Maintain your own trust-domain →
   trust-bundle/issuer mapping and verify against it.
4. **Confusables / lookalike trust domains.** The crate restricts the trust-domain character set
   (it's an ASCII subset — no IDNA, so no Unicode-confusable domains in the first place) and
   lowercases — but `examp1e.org` (with a digit "1") vs `example.org` is *valid syntax for two
   different trust domains*. Your trust-domain allowlist is what stops the lookalike; the parser
   can't know which one you meant. Allowlist trust domains; don't pattern-match them.
5. **Path-semantic assumptions.** A path being syntactically valid says nothing about whether
   `ns/<ns>/sa/<sa>` corresponds to a real namespace/service-account, or whether the `extra`
   segments mean what you think. `as_wimse_workload()` returns `None` for paths that don't fit that
   shape — `None` is "not a WIMSE-shaped workload path", *not* an error, and *not* a security
   verdict. Decide what your service does with non-WIMSE-shaped paths.
6. **Bounding the input.** The crate caps at the *spec* limits (2048 / 255 bytes). If your transport
   would let a caller hand you a 1 GB string, *you* bound it first (your HTTP body limits, your JWT
   size limits, your config-value limits). The crate's caps protect the parser; they don't replace
   your edge limits.
7. **Comparing IDs.** Compare *parsed* `SpiffeId`s, not raw strings — two different byte strings can
   be the same identity after canonicalisation (casing of the trust domain, percent-encoding in the
   path). `SpiffeId` implements `PartialEq`/`Eq`/`Hash` over the canonical form; use that. Never do
   security-relevant comparisons on the pre-parse string.
8. **Downstream re-encoding.** If you take a parsed `SpiffeId` and stick it somewhere with its own
   syntax — a SQL query, a log line a parser will read back, a URL path, a SpiceDB object id — apply
   *that* context's encoding/escaping. The crate gives you a clean canonical SPIFFE URI; making it
   safe in a *different* grammar is on you.

### A safe-usage checklist for this audience

- [ ] The bytes you pass to `spiffe-id` come from a source you've already *authenticated* (a
      verified token, a validated certificate, trusted config) — or you treat the result as
      *unauthenticated* and gate everything on a real authn step.
- [ ] You bound the input size at your edge before parsing.
- [ ] Authz keys off the structured components of the *authenticated* `SpiffeId`, exact-match, with
      a trust-domain allowlist — never a substring/regex match on the URI string.
- [ ] You map each accepted trust domain to a specific trust bundle / issuer and verify against it.
- [ ] You've decided what happens for `as_wimse_workload() == None` (non-WIMSE-shaped path) — it's a
      policy choice, not a default-deny the crate makes for you.
- [ ] You compare `SpiffeId` values, not strings.
- [ ] You re-encode for the destination grammar whenever you embed an ID somewhere new.

---

## Audience 2 — you write a crate/library that depends on (or re-exports) `spiffe-id`

- **Semver & MSRV are *your* surface now.** If you re-export `spiffe-id` types in your public API,
  your users feel its breaking changes; pin a caret range you're comfortable with and read the
  [`CHANGELOG.md`](../CHANGELOG.md) before bumping. The crate's MSRV is documented in `Cargo.toml`
  (`rust-version`); your effective MSRV is the max of yours and its.
- **Feature unification.** If you enable `spiffe-id`'s `serde` feature, every crate in the build
  graph that uses `spiffe-id` gets it (Cargo unifies features). That's usually fine (the feature
  only adds string (de)serialisation), but be deliberate — don't enable it "just in case" in a
  crate that doesn't need it.
- **Don't re-validate, don't re-loosen.** A `SpiffeId` you receive is already validated — don't
  re-parse it defensively (waste), and *definitely* don't accept a raw string "to be flexible" in a
  spot where you should require a `SpiffeId` (you'd be re-introducing the gap the crate closes).
- **`no_std`.** The crate is `no_std` + `alloc` in *every* configuration — there's no `std` feature,
  and `ParseError` impls `core::error::Error` (Rust ≥ 1.81) so it works in `no_std` `?`-chains too.
  Nothing to set for a `no_std` consumer; just don't enable `serde` unless you need it (it pulls in
  `serde` for the whole build graph — usually fine, but be deliberate).
- **Pass the [`ROADMAP.md`](../ROADMAP.md) caveat through.** This crate is *URI parsing only* — no
  SVID/document handling, no trust bundles, no Workload API. If your users might expect those from
  "the SPIFFE crate", say in your docs that you're using `spiffe-id` for the ID layer specifically.

---

## Audience 3 — the maintainers / the supply chain

Covered in detail by the playbooks; the summary of the threat surface:

- **The crate has no runtime dependencies and no build script** — so there is no transitive
  supply-chain blast radius and nothing executes at build time. The only ways a malicious change
  reaches users are: a bad commit/tag pushed to this repo, or a bad version published to crates.io
  under the `spiffe-id` name. Both require compromising a maintainer's GitHub account / crates.io
  token / the CI pipeline → [`docs/playbooks/maintainer-compromise.md`](playbooks/maintainer-compromise.md).
- **Typosquatting** — a *different* crate named close to `spiffe-id` (or `wimse`) that isn't ours.
  Mitigation: the README and docs state the canonical crate name and repo; we'd file a crates.io
  abuse report against a malicious typosquat. There's not much else to do — tell people where the
  real one is.
- **A latent parsing bug becoming a vulnerability** — the ordinary case, handled by
  [`docs/playbooks/vulnerability-response.md`](playbooks/vulnerability-response.md). Mitigations on
  our side: a strict spec-derived test suite, `#![forbid(unsafe_code)]`, `clippy -D warnings`, the
  full feature/target matrix in CI, and (planned) a fuzz target — see the roadmap.
- **A toolchain / `serde` issue** — out of our scope; we depend on RustSec / Dependabot to flag it
  and would pull a fix through promptly.

Maintainer hardening (2FA with hardware keys, scoped publish tokens, branch protection, signed
tags, no publish creds in CI, `cargo audit` in CI) is checklisted at the end of the
[maintainer-compromise playbook](playbooks/maintainer-compromise.md).

---

## Out of scope (restating, because it matters)

- **SVID parsing/validation** — X.509-SVID (the cert), JWT-SVID (the token). `spiffe-id` parses the
  *ID URI* those carry; it does not parse, verify signatures on, or check expiry of the documents.
- **Trust bundles / federation** — what trust domain maps to what set of keys, and federation
  policy between trust domains. Not here.
- **The SPIFFE Workload API** — fetching SVIDs from a SPIRE-style agent. Not here.
- **Network, time, randomness** — the crate touches none of them.
- **Authentication & authorization** — said three times above; it's the most common misunderstanding
  of "I'm using the SPIFFE crate, so I'm secure". You are using *a parser*.

If you need the document/bundle/Workload-API layers, see the [`spiffe`](https://crates.io/crates/spiffe)
crate (heavier; includes the Workload API client) — `spiffe-id` is the slim ID layer those build on,
and the place to look if all you need is "is this string a valid SPIFFE ID, and what are its parts".
