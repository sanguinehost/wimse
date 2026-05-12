# Contributing to `spiffe-id`

Thanks for looking. This is a small, deliberately-narrow crate — a strict parser for SPIFFE / WIMSE
workload-identity URIs — so the bar for changes is "does it make the parsing more correct, the API
clearer, or the project healthier, without expanding the scope?" Bug reports, missing test vectors,
docs fixes, and small focused PRs are all very welcome.

## Before you start

- **Security issues:** do **not** open a public issue/PR. See [`SECURITY.md`](SECURITY.md) — report
  privately to `security@twn.systems` or via GitHub's private vulnerability reporting.
- **Big changes / new scope:** open an issue (or a Discussion) first. The crate intentionally does
  *not* do SVID documents, trust bundles, or the Workload API (see [`ROADMAP.md`](ROADMAP.md) for
  the why and what's actually planned) — a PR adding those will be declined. A PR adding a small,
  well-justified accessor or fixing a spec corner won't need a prior issue.
- **Behaviour of `acknowledged`:** this project follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## Dev setup

You need a recent stable Rust — the MSRV is pinned in `Cargo.toml`'s `rust-version` (currently
**1.81**, for `core::error::Error`). Then:

```bash
git clone https://github.com/sanguinehost/wimse
cd wimse

# the full local gate (this is what CI runs):
cargo fmt --check
cargo clippy --all-targets --features serde -- -D warnings
cargo test                       # default features
cargo test --no-default-features # no_std + alloc only
cargo test --features serde
cargo check --target wasm32-unknown-unknown   # add the target: rustup target add wasm32-unknown-unknown
cargo doc --no-deps              # docs build (optional locally; CI checks it)
```

If `cargo fmt --check`, `cargo clippy … -D warnings`, and the three `cargo test` invocations are
green, your change will almost certainly pass CI. The wasm32 check just needs the target installed.

## What a good PR looks like

- **One thing.** A bug fix *or* a new accessor *or* a docs improvement — not a grab-bag. Small PRs
  get reviewed fast.
- **Tests for the change.** A bug fix comes with a regression test that fails before and passes
  after. New behaviour comes with tests for the happy path *and* the rejection cases. The test
  suite is the spec compliance evidence — keep it that way. If you found a SPIFFE-spec corner the
  crate gets wrong, a failing test is the best possible bug report.
- **`#![forbid(unsafe_code)]` stays.** No `unsafe`, ever, in this crate.
- **`no_std` stays.** Don't reach for `std` — the crate is `#![no_std]` + `alloc`. If you need an
  allocation, use `alloc::{String, Vec, …}`; if you need to `format!`, it's `alloc::format!`.
- **Zero runtime deps stays.** The only optional dependency is `serde` (behind the `serde` feature),
  and dev-deps are `serde_json`. A PR adding a new dependency needs a very good reason and a prior
  discussion.
- **Public API changes are semver-relevant.** Adding an item is a minor bump; changing/removing one
  is a breaking (major, pre-1.0: minor) bump. Note it in the PR; we'll update `CHANGELOG.md` and the
  version on release.
- **Docs.** Public items have rustdoc (the crate denies `missing_docs` — sorry, but it keeps docs.rs
  useful). If you change behaviour, update the relevant prose in `README.md` / `docs/`.
- **Clippy is `-D warnings`.** Including `pedantic`. If a lint is genuinely wrong for a spot, a
  narrow `#[allow(...)]` with a one-line reason is fine; a blanket allow isn't.

## Commit & PR conventions

- **Commit messages:** [Conventional Commits](https://www.conventionalcommits.org/) —
  `feat: …`, `fix: …`, `docs: …`, `test: …`, `refactor: …`, `chore: …`, `ci: …`, `perf: …`. Subject
  ≤ 72 chars, imperative mood. A body explaining *why* (not just what) is appreciated for anything
  non-trivial.
- **AI-assisted commits:** if a commit was written with substantial help from an AI coding tool, add
  a `Co-Authored-By:` trailer for it (matching the rest of this workspace's convention).
- **Sign-off (DCO):** add `Signed-off-by: Your Name <your@email>` to your commits
  (`git commit -s`) — by doing so you certify the [Developer Certificate of Origin](https://developercertificate.org/)
  (you wrote it / have the right to submit it / it's under the project's licence). There's no
  separate CLA.
- **PRs:** target `main`; fill in the PR template (what / why / how it's tested); link the issue if
  there is one. Keep the branch rebased on `main`. CI must be green. A maintainer will review — be
  patient; this is a small team.

## Licence

By contributing you agree your contribution is licensed under the project's licence ([MIT](LICENSE)) —
the DCO sign-off above is how you assert that.

## Releasing (maintainers)

1. Confirm the full gate is green on `main` (the CI matrix).
2. Bump `version` in `Cargo.toml`; add the `CHANGELOG.md` entry under a new version heading.
3. `cargo publish --dry-run` from a clean checkout; eyeball the file list.
4. `git tag vX.Y.Z && git push origin main --tags`.
5. `cargo publish` (with a crates.io token **scoped to `spiffe-id`, publish-only**, from a trusted
   machine — never from CI; see the [maintainer-compromise playbook](docs/playbooks/maintainer-compromise.md)
   for why).
6. Verify the version on <https://crates.io/crates/spiffe-id> and that <https://docs.rs/spiffe-id>
   built. Cut a GitHub Release from the tag with the changelog excerpt.
7. For a *security* release, follow [`docs/playbooks/vulnerability-response.md`](docs/playbooks/vulnerability-response.md)
   instead — the fix is developed privately and the release/disclosure order matters.

## Questions

Open a [Discussion](https://github.com/sanguinehost/wimse/discussions) or a non-security issue.
