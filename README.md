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
scope (later iterations / other crates).

## Related repos

- [`sanguinehost/ferrousgate`](https://github.com/sanguinehost/ferrousgate) — MCP gateway; home of the iter-18 spec
- [`sanguinehost/aim`](https://github.com/sanguinehost/aim) — agent IdP (consumes this crate)
- [`sanguinehost/agent-identity-resolver`](https://github.com/sanguinehost/agent-identity-resolver) — identity resolver (iter-18b)
