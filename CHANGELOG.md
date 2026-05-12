# Changelog

Notable changes to the `spiffe-id` crate. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
the crate follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html) (pre-`1.0`: a `0.x`
minor bump may be breaking).

## [Unreleased]

### Added
- Project documentation: `SECURITY.md` (report to `security@twn.systems`), `CONTRIBUTING.md`,
  `CODE_OF_CONDUCT.md`, `GOVERNANCE.md`, `ROADMAP.md`, `FEATURES.md`; `docs/` (threat model for three
  audiences, integration guide, implementation guide) and `docs/playbooks/` (the GHSA /
  vulnerability-response playbook and the maintainer-compromise playbook); `.github/` issue & PR
  templates, `CODEOWNERS`, `dependabot.yml`.
- `Cargo.toml`: declared `rust-version = "1.81"` (the de-facto MSRV — `core::error::Error`).

## [0.1.0] — 2026-05-10

Initial release — the `spiffe-id` crate, implementing the `iter-18-spiffe-id.md` spec from
[`sanguinehost/ferrousgate`](https://github.com/sanguinehost/ferrousgate).

### Added
- `SpiffeId` — a typed, validated SPIFFE ID. `SpiffeId::parse(&str) -> Result<SpiffeId, ParseError>`
  (also `FromStr`): validates the scheme (`spiffe`, case-insensitive on input), the trust-domain
  authority (non-empty, ≤ 255 bytes, allowed character set, no userinfo, no port), and the path
  (`/`-separated, non-empty / non-`.`/`..` segments, allowed characters, well-formed percent-encoding,
  no trailing slash); rejects query/fragment; enforces the 2048-byte total cap. Canonicalises:
  lowercases the scheme and trust domain, percent-decodes the path. Accessors: `trust_domain()`,
  `path()`, `path_segments()`, `in_trust_domain()`, `as_uri()` (round-trips). `Display`, `Clone`,
  `Debug`, `PartialEq`/`Eq`/`Hash` (over the canonical form).
- `as_wimse_workload() -> Option<WimseWorkloadId<'_>>` — the structured
  `ns/<namespace>/sa/<service-account>[/<extra>...]` workload view; `None` for non-matching paths
  (not an error); never errors.
- `ParseError` — a `match`-able enum (`BadScheme`, `DisallowedComponent`, `BadTrustDomain`,
  `BadPath`, `TooLong`, `Malformed`); `Display` + `core::error::Error`.
- `#![no_std]` + `alloc`, `#![forbid(unsafe_code)]`, zero runtime dependencies; builds for
  `wasm32-unknown-unknown`. Optional `serde` feature: `Serialize`/`Deserialize` for `SpiffeId` as
  the canonical URI string, deserialising by `parse`.
- A test suite mirroring the spec's "Test plan" (`tests/spiffe_id.rs`) + a doc-test; CI running
  check / `--no-default-features` / `wasm32` / `--features serde` test / clippy / fmt.

[Unreleased]: https://github.com/sanguinehost/wimse/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sanguinehost/wimse/releases/tag/v0.1.0
