# wimse — `spiffe-id`

**SPIFFE / WIMSE URI parsing library. Zero runtime deps, `no_std`-friendly.
Consumed by AIM and the agent-identity-resolver.**

This repo hosts the `spiffe-id` crate: a typed `SpiffeId` newtype that parses
and validates [SPIFFE-ID](https://github.com/spiffe/spiffe/blob/main/standards/SPIFFE-ID.md)
URIs (`spiffe://<trust-domain>/<path>`) and the
[WIMSE](https://datatracker.ietf.org/wg/wimse/about/) workload-identifier
grammar (`ns/<namespace>/sa/<service-account>[/<extra>...]`).

| Field | Value |
|---|---|
| Crate | `spiffe-id` |
| Spec | `docs/specs/iter-18-spiffe-id.md` in [`sanguinehost/ferrousgate`](https://github.com/sanguinehost/ferrousgate) |
| Iteration | iter-18 |
| License | MIT |
| `no_std` | yes — `core` + `alloc` only, no I/O, no crypto (so `aim-wasm` can use it) |

## Usage

```rust
use spiffe_id::SpiffeId;

let id = SpiffeId::parse("spiffe://twn.network/ns/prod/sa/agent-orchestrator")?;
assert_eq!(id.trust_domain(), "twn.network");
assert_eq!(id.path(), "ns/prod/sa/agent-orchestrator");

// WIMSE workload view (None for non-WIMSE paths — not an error):
let wl = id.as_wimse_workload().unwrap();
assert_eq!(wl.namespace, "prod");
assert_eq!(wl.service_account, "agent-orchestrator");

// Round-trips back to the canonical URI:
assert_eq!(id.as_uri(), "spiffe://twn.network/ns/prod/sa/agent-orchestrator");
# Ok::<(), spiffe_id::ParseError>(())
```

`Display` / `FromStr` are implemented; enable the `serde` feature to
(de)serialise as the URI string.

## Features

- `serde` (off by default) — `Serialize` as the URI string, `Deserialize` via
  `SpiffeId::parse`.

## What this crate does *not* do

ID-URI parsing only. SVID document parsing (X.509-SVID, JWT-SVID), trust-domain
bundle management, the SPIFFE Workload API, and federation policy are all out of
scope (other crates — see [`ROADMAP.md`](ROADMAP.md), and [`spiffe`](https://crates.io/crates/spiffe)
for the full stack). **And — important — a valid SPIFFE ID is not authentication
or authorization**; the trust domain in an ID is whatever string was in the input,
not a verified issuer. See the [threat model](docs/THREAT-MODEL.md).

## Documentation

The API reference is the rustdoc ([docs.rs/spiffe-id](https://docs.rs/spiffe-id)). Around it:

- [`FEATURES.md`](FEATURES.md) — what it does: the validation rules, the accessors, the error type, the `serde` feature, the properties.
- [`docs/THREAT-MODEL.md`](docs/THREAT-MODEL.md) — what it defends against, what it doesn't, what's on you — for service operators, dependent-crate authors, and maintainers. **Read before shipping.**
- [`docs/INTEGRATION.md`](docs/INTEGRATION.md) — taking the dependency, the parse-at-the-boundary pattern, wasm/embedded, where it fits in a SPIFFE pipeline, the supply-chain story.
- [`docs/IMPLEMENTATION.md`](docs/IMPLEMENTATION.md) — how it works inside: the parse pipeline, the canonical-form invariant, the `no_std` strategy, the test strategy.
- [`ROADMAP.md`](ROADMAP.md) — what's done, what's next, and what's deliberately out of scope (and why).
- [`docs/README.md`](docs/README.md) — the index of all of the above.

### Project & security

- [`SECURITY.md`](SECURITY.md) — report a vulnerability privately (**`security@twn.systems`** or GitHub private vulnerability reporting); scope, supported versions, disclosure process, safe harbour.
- [`docs/playbooks/`](docs/playbooks/) — maintainer runbooks: the [GHSA / vulnerability-response playbook](docs/playbooks/vulnerability-response.md) and the [maintainer-compromise playbook](docs/playbooks/maintainer-compromise.md).
- [`CONTRIBUTING.md`](CONTRIBUTING.md) · [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) · [`GOVERNANCE.md`](GOVERNANCE.md) · [`CHANGELOG.md`](CHANGELOG.md).
