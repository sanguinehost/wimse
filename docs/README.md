# `spiffe-id` — docs

The longer-form docs for the [`spiffe-id`](../README.md) crate (`sanguinehost/wimse`). The API
reference proper is the rustdoc on [docs.rs](https://docs.rs/spiffe-id); this tree is the *context*
around it — what it does, why the scope is what it is, how it works inside, how to use it safely,
and the maintainer runbooks.

## Map

| Doc | What it's for | Audience |
|---|---|---|
| [`../README.md`](../README.md) | The 60-second overview + a usage snippet. | everyone |
| [`../FEATURES.md`](../FEATURES.md) | What the crate does — the validation rules, the accessors, the error type, the `serde` feature, the "non-functional" properties (`no_std`, zero-dep, strict, spec-tested), and what it explicitly is *not*. | consumers deciding whether/how to use it |
| [`../ROADMAP.md`](../ROADMAP.md) | What's done, what's likely next (additive `0.x`), the path to `1.0`, and — the long part — what's **deliberately out of scope** (SVID documents, trust bundles, the Workload API, IDNA, anything I/O) and why. | consumers, contributors, "should this crate also do X?" |
| [`THREAT-MODEL.md`](THREAT-MODEL.md) | The honest version: what the crate defends against, what it doesn't, and what's therefore on you — written for three audiences (a service parsing untrusted IDs; a crate that depends on this one; the maintainers / supply chain). **Read this before you ship `spiffe-id` in anything that makes security decisions.** | service operators, dependent-crate authors, maintainers |
| [`INTEGRATION.md`](INTEGRATION.md) | The "deployment" guide for a library: taking the dependency, version pinning, the parse-at-the-boundary pattern, wiring it into a service / wasm / embedded, where it sits in a SPIFFE/WIMSE pipeline, and the supply-chain story. | consumers integrating it |
| [`IMPLEMENTATION.md`](IMPLEMENTATION.md) | How it works inside: the parse pipeline step by step, the data layout & canonical-form invariant, the `no_std` strategy, the test strategy, the specs it implements, and the rules for extending it without breaking anything. | contributors, the curious |
| [`playbooks/`](playbooks/) | Maintainer runbooks: the [vulnerability-response / GHSA playbook](playbooks/vulnerability-response.md) and the [maintainer-compromise playbook](playbooks/maintainer-compromise.md) (with a hardening checklist). | maintainers |
| [`../SECURITY.md`](../SECURITY.md) | The public security policy — how to report a vulnerability (to `security@twn.systems` or GitHub private reporting), scope, supported versions, the disclosure process, safe harbour. | reporters, users |
| [`../CONTRIBUTING.md`](../CONTRIBUTING.md) | Dev setup, the local gate (= CI), what a good PR looks like, commit/DCO conventions, the release checklist. | contributors |
| [`../CODE_OF_CONDUCT.md`](../CODE_OF_CONDUCT.md), [`../GOVERNANCE.md`](../GOVERNANCE.md) | The community standards and who-decides-what. | everyone / contributors |

## "I just want to…"

- **…use the crate** → [`../README.md`](../README.md) snippet, then [`FEATURES.md`](../FEATURES.md)
  for the full API, then [`THREAT-MODEL.md`](THREAT-MODEL.md) §"Audience 1" *before you ship*, and
  [`INTEGRATION.md`](INTEGRATION.md) for the wiring.
- **…report a bug** → a [GitHub issue](https://github.com/sanguinehost/wimse/issues) (use the
  template). **A security bug?** → [`../SECURITY.md`](../SECURITY.md), *not* a public issue.
- **…contribute** → [`../CONTRIBUTING.md`](../CONTRIBUTING.md).
- **…understand why it won't do `<X>`** → [`ROADMAP.md`](../ROADMAP.md) "out of scope", and the
  scope notes in [`IMPLEMENTATION.md`](IMPLEMENTATION.md).
- **…run a security incident (you're a maintainer)** → [`playbooks/`](playbooks/).
