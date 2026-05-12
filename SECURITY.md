# Security Policy

`spiffe-id` (this repository, `sanguinehost/wimse`) is a parsing library that sits on a trust
boundary — services use it to validate identifiers that arrive from untrusted callers. We take
reports about it seriously and aim to respond fast.

## Reporting a vulnerability

**Please report security issues privately. Do not open a public GitHub issue, discussion, or pull
request for anything that could be a vulnerability.**

Two private channels — use whichever is easier:

1. **Email** — **`security@twn.systems`**.
   - Subject line: `[security] spiffe-id: <short description>`.
   - Include: what you found, where (commit / version / file:line if you can), how to reproduce
     (a minimal `cargo test`-able snippet is ideal), the impact you think it has, and any
     mitigations you've identified.
   - If you need to encrypt: say so in a first (non-sensitive) email and we'll exchange a PGP key,
     or just use channel 2 below (TLS-protected end to end).
2. **GitHub private vulnerability reporting** — on this repo, *Security → Advisories → Report a
   vulnerability* (<https://github.com/sanguinehost/wimse/security/advisories/new>). This opens a
   private thread between you and the maintainers and is the channel we use to draft the eventual
   GHSA.

You'll get an acknowledgement within **2 business days**. If you don't, please re-send — mail does
occasionally go astray.

## What's in scope

- The `spiffe-id` crate published from this repository, all supported versions (see below).
- Anything that lets a caller make `spiffe-id` accept an identifier it should reject, reject one it
  should accept, mis-report a component (trust domain / path / WIMSE workload view), or that causes
  unbounded resource use (CPU, memory) on a *bounded* input within the documented length caps.
- The build / release / publish pipeline for this crate (CI configuration, the crates.io release,
  the repository's branch protection and required checks) — i.e. supply-chain integrity. See the
  [maintainer-compromise playbook](docs/playbooks/maintainer-compromise.md).

## What's out of scope

- **Misuse of a *valid* identifier.** A SPIFFE ID being syntactically valid is not authentication or
  authorization. The trust domain in an ID is **not** a verified issuer — it is whatever string was
  in the input. Treating a parsed ID as a grant is a bug in the *consumer*, not this crate. (We say
  this loudly in the [threat model](docs/THREAT-MODEL.md) and the docs.)
- SVID document handling (X.509-SVID, JWT-SVID), trust-bundle / federation logic, the SPIFFE
  Workload API — not implemented here (out of scope for the crate; see [`ROADMAP.md`](ROADMAP.md)).
- Vulnerabilities in your code, your dependency tree (other than this crate), the Rust toolchain,
  `cargo`, or `serde`/`serde_json` (report those to their respective projects).
- Findings that require an attacker to already control the parsing process (e.g. "if you `unsafe`ly
  transmute a `SpiffeId`…") — the crate is `#![forbid(unsafe_code)]`; we treat any actual UB as a
  critical bug, but contrived "if you break the type system first" reports are not.
- Denial of service from inputs *larger* than the documented caps when the caller didn't bound the
  input — bound your inputs (the crate enforces the SPIFFE-spec 2048-byte / 255-byte caps; if you
  hand it a gigabyte string, that allocation is on you). Reports of *super-linear* time/memory in the
  parser on bounded input *are* in scope.

## Supported versions

Until `1.0`, the latest `0.x` minor is supported; security fixes go out as a new patch on that
minor, and (best-effort) as a backport to the previous minor for ~90 days.

| Version | Supported |
|---|---|
| `0.1.x` (latest) | ✅ |
| older `0.x` | ❌ — upgrade |

After `1.0`: the current major, plus security backports to the previous major for 12 months.

## Our disclosure process (coordinated disclosure)

We follow coordinated disclosure with a default **90-day** window from first report to public
disclosure (we'll usually move faster). The full lifecycle — triage, CVSS scoring, drafting the
GHSA, requesting a CVE, building the fix, the coordinated release, the [RustSec](https://rustsec.org/)
advisory, and the post-mortem — is documented in
[`docs/playbooks/vulnerability-response.md`](docs/playbooks/vulnerability-response.md). In short:

1. **Acknowledge** within 2 business days; agree a private channel.
2. **Triage & reproduce**; assign a severity (CVSS v3.1) and a target fix date.
3. **Fix in private** (a private fork / temporary private branch), with a regression test.
4. **Coordinate**: agree a disclosure date with you (and with major downstreams if the impact is
   broad); request a CVE via the GitHub Security Advisory.
5. **Release**: publish the patched version(s) to crates.io, then publish the GHSA, file the
   [RustSec advisory](https://github.com/rustsec/advisory-db), and `cargo yank` affected versions
   if warranted.
6. **Post-mortem**: a public write-up of what happened and what changed, with credit to the
   reporter (unless you ask otherwise).

## Recognition

We will credit you in the advisory and the changelog (name and/or handle and/or link, your choice;
or anonymously). We don't run a paid bug-bounty program for this crate — but a clear, well-written
report on a real issue is genuinely appreciated and we'll say so publicly.

## Safe harbour

If you make a good-faith effort to follow this policy — report privately, don't access data that
isn't yours, don't disrupt services, give us a reasonable time to fix before disclosing — we will
not pursue or support legal action against you for that research, and we'll treat it as authorised.
Good faith is judged on the whole of your conduct; this is not a licence to do whatever you want.

---

*Maintained by [TWN Systems](https://twn.systems) / [Sanguine Host](https://github.com/sanguinehost).
Security contact: `security@twn.systems`. Code-of-conduct contact: `conduct@twn.systems`.*
