# Governance

`spiffe-id` (this repo, `sanguinehost/wimse`) is a small open-source library maintained by
[TWN Systems](https://twn.systems) under the [Sanguine Host](https://github.com/sanguinehost)
organisation. This document says who decides what, and how — kept light, because the project is
small and the scope is narrow.

## Who maintains it

- **Maintainers** — people with write access to this repo, who review and merge PRs, cut releases,
  and run security incidents. Currently: the Sanguine Host / TWN Systems engineering team, with
  `@yokoszn` (Clay Townsend) as the lead maintainer / default decision-tiebreaker.
- **Contributors** — anyone who's had a PR merged. (Thank you.)
- **Code owners** — listed in [`.github/CODEOWNERS`](.github/CODEOWNERS); a PR needs review from a
  code owner before it lands.

## How decisions are made

- **Day-to-day** (bug fixes, small accessors, docs, test additions): a maintainer review + green CI
  is enough. Lazy consensus — if no maintainer objects, it goes in.
- **Scope changes** (anything in the "out of scope" list in [`ROADMAP.md`](ROADMAP.md), a new
  dependency, a breaking API change): discuss first (an issue or a Discussion); needs explicit
  agreement from the lead maintainer (or, if the lead is unavailable, two maintainers). The default
  answer to "should this crate also do X?" is *no* — the smallness is a feature.
- **Releases:** any maintainer may cut a routine release following the checklist in
  [`CONTRIBUTING.md`](CONTRIBUTING.md#releasing-maintainers); a *security* release follows
  [`docs/playbooks/vulnerability-response.md`](docs/playbooks/vulnerability-response.md) and the
  release/disclosure order matters. The crates.io owner of `spiffe-id` is the org / the lead
  maintainer; publish tokens are scoped per-maintainer (see the
  [maintainer-compromise playbook](docs/playbooks/maintainer-compromise.md)).
- **Conflicts:** the lead maintainer decides, after hearing it out. We'd rather not exercise that —
  the scope is small enough that there's not much to fight about.

## Becoming a maintainer

There's no committee process. If you've contributed substantively over time, understand the SPIFFE/
WIMSE specs and the crate's "stay small" philosophy, and a current maintainer proposes it (and the
lead agrees), you're in. We'll add you to `CODEOWNERS`, give you write access, and issue you a
scoped crates.io publish token. New maintainers read the playbooks and do the
[hardening checklist](docs/playbooks/maintainer-compromise.md#hardening-checklist-do-these-now-before-an-incident)
(2FA with a hardware key, etc.) before they get publish rights.

## Stepping down / inactivity

Maintainers can step down any time — open a PR removing yourself from `CODEOWNERS` (and tell the
lead so the access/token cleanup happens). If a maintainer is unreachable for a long time, the lead
may remove their write access and revoke their tokens as a precaution (it's reversible).

## Ownership & licence

The repo and the `spiffe-id` crate name are owned by TWN Systems / Sanguine Host. The code is
[MIT-licensed](LICENSE); contributions are accepted under that licence via the DCO sign-off (see
[`CONTRIBUTING.md`](CONTRIBUTING.md)). If the project were ever to be archived or transferred, that
decision is the org's, announced in the README and a release note, with maintainers given notice.

## Code of Conduct

Everyone participating — maintainers, contributors, issue/PR/Discussion participants — is bound by
the [Code of Conduct](CODE_OF_CONDUCT.md). Enforcement reports go to `conduct@twn.systems` (a
different mailbox from the `security@twn.systems` vulnerability channel — keep the two separate).

## Changing this document

Like everything else: a PR, a maintainer review. Substantive governance changes get a heads-up in a
Discussion first.
