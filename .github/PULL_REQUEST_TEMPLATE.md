<!--
Thanks for the PR! A few things before you submit (delete this comment when done):

• SECURITY ISSUES: do NOT use a PR. See SECURITY.md — report privately to security@twn.systems
  or via GitHub's private vulnerability reporting.
• Big changes / new scope: please open an issue or Discussion first. The crate intentionally does
  NOT do SVID documents, trust bundles, or the Workload API — see ROADMAP.md "out of scope".
• Run the local gate (= CI):
    cargo fmt --check
    cargo clippy --all-targets --features serde -- -D warnings
    cargo test  &&  cargo test --no-default-features  &&  cargo test --features serde
    cargo check --target wasm32-unknown-unknown
• `#![forbid(unsafe_code)]`, `#![no_std]` + `alloc`, and zero new runtime deps all stay.
• Conventional Commits (feat: / fix: / docs: / test: / refactor: / chore: / ci: / perf:);
  sign off your commits (`git commit -s` — the DCO; see CONTRIBUTING.md). AI-assisted commit?
  add a `Co-Authored-By:` trailer for the tool.
-->

## What & why

<!-- What does this change, and why? Link the issue if there is one (`Closes #…`). -->

## How it's tested

<!-- New/changed behaviour ⇒ tests. A bug fix ⇒ a regression test that fails before, passes after.
     Note any spec corner this covers. Which `cargo test` invocations did you run? -->

## Checklist

- [ ] One focused change (a fix *or* a small accessor *or* docs — not a grab-bag).
- [ ] Tests added/updated for the change; `tests/spiffe_id.rs` still mirrors the spec where relevant.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets --features serde -- -D warnings`, and the
      `cargo test` matrix (default / `--no-default-features` / `--features serde`) are green locally.
- [ ] `cargo check --target wasm32-unknown-unknown` is green (still `no_std`).
- [ ] No `unsafe`, no `std`, no new runtime dependency.
- [ ] Public API change? Noted here (it's semver-relevant) and the rustdoc/`CHANGELOG.md` is updated.
      Behaviour change? The relevant prose in `README.md` / `docs/` is updated.
- [ ] Public items have rustdoc (the crate denies `missing_docs`).
- [ ] Commits follow Conventional Commits and are signed off (DCO).
- [ ] This is **not** a security issue (those go to `security@twn.systems`, not a PR).
