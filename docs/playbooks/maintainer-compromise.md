# Playbook — maintainer / supply-chain compromise

**Audience:** maintainers of `sanguinehost/wimse` (the `spiffe-id` crate), and anyone helping
respond to a suspected compromise. **When to use:** you have reason to believe a maintainer's GitHub
account, their crates.io API token, a signing key, or the CI pipeline for this repo has been — or
might have been — taken over, and that the attacker could push code, publish a release, or alter the
build. This is the "the supply chain might be poisoned" runbook. It's deliberately bias-to-action:
**contain first, investigate second.**

Cross-reference: if a *malicious version actually shipped to crates.io*, you'll also run the
advisory half of [`vulnerability-response.md`](vulnerability-response.md) (CVE + GHSA + RustSec).

> **A note on this crate specifically.** `spiffe-id` has **zero runtime dependencies** and no build
> script — so the *only* ways a supply-chain attack lands are (a) a malicious commit/tag pushed to
> this repo, or (b) a malicious version published to crates.io under the `spiffe-id` name (which
> needs a crates.io owner token), or (c) a typosquat crate that *isn't* this one. (a) and (b) are
> what this playbook covers; (c) is "tell people the canonical name and link". There is no
> transitive-dependency blast radius to worry about — which is one reason the crate is zero-dep.

---

## 0. Triggers — any of these starts this playbook

- A maintainer reports their GitHub or crates.io account did something they didn't do (a login from
  an unknown location, a push/tag/release they didn't make, a token they didn't create, a 2FA reset
  email they didn't request).
- A `spiffe-id` version appears on crates.io that no maintainer recall publishing, or a published
  version's contents don't match this repo at the corresponding tag.
- A commit/tag on `main` that no maintainer made (and not a legitimate bot).
- GitHub or crates.io security/abuse contacts you about this repo or a maintainer.
- A maintainer's laptop / dev environment is known-compromised (malware, stolen, lost unlocked).
- A leaked credential is found in CI logs, an issue, a gist, a pastebin, etc.

If you're *not sure* — run it anyway. Containment is cheap and reversible; a poisoned release in the
wild is not.

---

## 1. Contain — first 60 minutes, in roughly this order

Do the ones you have access to; ask another maintainer (or GitHub/crates.io support) for the rest.

### crates.io (the publish path — highest priority)

- [ ] **Revoke all crates.io API tokens** for every maintainer with publish rights:
  <https://crates.io/settings/tokens> → revoke. Do this even for tokens you think are fine — it's a
  few clicks and you re-mint scoped ones later.
- [ ] **Check the published versions** of `spiffe-id` at <https://crates.io/crates/spiffe-id> →
  *Versions*. If there's a version you didn't publish, or a version whose `.crate` contents don't
  match this repo at its tag (compare with `cargo package --list` / download the `.crate` and diff):
  `cargo yank --version X.Y.Z spiffe-id` **immediately** — yanking stops new dependents from picking
  it up. (You cannot delete a crates.io version; yank is the lever. Note: existing `Cargo.lock`s
  that already pinned the bad version keep using it — that's why a GHSA/RustSec advisory matters
  too.)
- [ ] **Check the crate owners** at the bottom of the crates.io page — if an unexpected GitHub user
  or team has been added as an owner, remove them (`cargo owner --remove <user> spiffe-id`) if you
  still can; if you can't, escalate to crates.io support (`help@crates.io`) marking it urgent.

### GitHub (the repo & accounts)

- [ ] **Lock the repo's default branch.** Settings → Branches → branch protection on `main`:
  require PR + review, no force-push, no deletions, restrict who can push (to just the trusted
  maintainers), require status checks. If it's already protected, tighten "who can push" to the
  minimum and disable any "allow administrators to bypass" toggle.
- [ ] **Compromised maintainer's account:** they should immediately — change the password, reset
  GitHub 2FA (re-enroll the authenticator/security key, regenerate recovery codes), and review
  *Settings → Sessions* and *Settings → Applications* (revoke any session/OAuth-app/GitHub-App/PAT
  they don't recognise, and any SSH/GPG keys they don't recognise). If they can't get in, GitHub
  account recovery + GitHub Support (mark it security).
- [ ] **Remove the compromised account's write access** to this repo (Settings → Collaborators &
  teams) until it's confirmed clean — and from the `sanguinehost` org if it's an org member.
- [ ] **Rotate every secret the repo's GitHub Actions can see** (Settings → Secrets and variables →
  Actions): the crates.io publish token (`CARGO_REGISTRY_TOKEN` or similar), any `GH_PAT`, anything
  else. Assume the attacker exfiltrated them via a CI run. (This crate's CI shouldn't *have* a
  publish token in normal operation — releases are done locally — but check.)
- [ ] **Pause CI** if a workflow itself looks tampered with: disable Actions for the repo (Settings →
  Actions → Disable), or set `workflow_dispatch`-only, until you've reviewed every workflow file.
- [ ] **Snapshot the evidence before you change more:** clone the repo with `--mirror`, screenshot
  the crates.io versions/owners pages, save the GitHub audit log (Org → Settings → Audit log) and
  the maintainer's account *Security log*, save any suspicious CI run logs. You'll want these for
  the post-mortem and any law-enforcement/insurance angle.

### Dev environment

- [ ] If a maintainer's machine is suspected compromised: take it offline, treat all credentials
  that ever touched it as burned (re-rotate even if you rotated above), and don't publish anything
  from it again until it's been wiped/reimaged.

---

## 2. Assess the blast radius — what could the attacker have done?

Work out the **window** (earliest plausible compromise → containment) and what was reachable in it:

- [ ] **Was anything published to crates.io in the window?** List `spiffe-id` versions and their
  publish timestamps; for any in-window version, download the `.crate`
  (`https://static.crates.io/crates/spiffe-id/spiffe-id-X.Y.Z.crate`), extract it, and diff against
  this repo at the matching tag. A mismatch (or no matching tag, or a tag you didn't create) = a
  poisoned release → it gets yanked (done in §1) **and** a CVE/GHSA/RustSec advisory (run
  [`vulnerability-response.md`](vulnerability-response.md)).
- [ ] **Was the repo modified in the window?** `git fetch --all && git log --all --since=<window>`,
  check the *Branches*, *Tags*, *Releases*, and the *Insights → Network* graph for anything you
  didn't do. Check force-pushes via the GitHub *Activity* / *Audit log*. A malicious commit on
  `main` that wasn't released is still bad (someone could `git pull` it) — but it's recoverable
  (§3).
- [ ] **Were CI secrets exposed?** If any in-window workflow run could have printed/leaked a secret
  (a malicious workflow change, a fork PR that touched the workflow — shouldn't be possible with
  proper `pull_request_target` hygiene, but check), treat those secrets as exfiltrated (re-rotate).
- [ ] **Was anything *else* in the org reachable?** If the compromised account had access to other
  `sanguinehost` repos, those need their own version of this playbook.
- [ ] **Write the window and the findings down** in the (private) incident doc — this is the spine
  of the post-mortem.

---

## 3. Recover

- [ ] **crates.io:** all bad versions yanked (done). If a *good* version needs re-publishing (e.g.
  you yanked the latest because it was tampered, but `main` is clean), bump the patch version, build
  from a *clean* checkout on a *clean* machine with a *freshly minted, scoped* token, and publish —
  then re-add the trusted owners only.
- [ ] **The repo:** if `main` (or a tag) was tampered with, restore it from a known-good commit. If
  that means rewriting history (force-pushing `main` back to a clean SHA), do it deliberately:
  announce it, since everyone with a clone must `git fetch && git reset --hard origin/main`. Re-tag
  if a tag was forged. Delete malicious branches/tags/releases.
- [ ] **Re-mint credentials minimally:** new crates.io tokens, **scoped to `spiffe-id` only** and to
  publish (not full account), one per maintainer, stored only where they need to be (a maintainer's
  machine, not in CI unless CI actually publishes). New GitHub PATs only if needed, fine-grained,
  scoped to this repo.
- [ ] **Re-grant access deliberately:** only the maintainers you've confirmed are clean get write
  back. Consider requiring `sanguinehost` org members to use hardware security keys for 2FA.
- [ ] **Re-enable CI** once every workflow file is reviewed and confirmed unchanged (or fixed).
- [ ] **Restore branch protection** to the tightened settings (require PR + ≥1 review, required
  status checks, no force-push, no admin bypass, restricted pushers, require signed commits if you
  adopt that).

---

## 4. Notify

- [ ] **GitHub** — if a GitHub account/repo was compromised, GitHub Support / `security@github.com`
  (they can help with account recovery and may have telemetry).
- [ ] **crates.io** — `help@crates.io` if a token/owner was abused or a malicious version shipped;
  they can assist with owners and may have publish telemetry.
- [ ] **RustSec** — if a malicious or backdoored version was published, a `RUSTSEC-YYYY-NNNN`
  advisory (as part of [`vulnerability-response.md`](vulnerability-response.md)) so `cargo audit` /
  `cargo deny` users are warned; RustSec also has a category for compromised/backdoored crates.
- [ ] **Downstream consumers** — the ones in the README (`sanguinehost/aim`,
  `sanguinehost/agent-identity-resolver`) and any you know of: "a version of `spiffe-id` in range
  X..Y may be compromised; pin to ≥ Z built from tag vZ; here's how to verify". Plain, fast, factual.
- [ ] **The reporter / whoever flagged it** — keep them in the loop.
- [ ] **`security@twn.systems`** gets the running incident log regardless.
- [ ] Consider whether disclosure obligations apply (if user data was involved — for a parsing
  library it usually isn't, but check).

---

## 5. Post-mortem (within 2 weeks)

Same shape as the [vulnerability-response post-mortem](vulnerability-response.md#6-post-mortem-within-2-weeks-of-disclosure):
what happened, the window, the blast radius, root cause (how did the credential/account get taken?
phishing? leaked token? reused password? no 2FA?), what we did, what we changed so it can't happen
the same way (hardware 2FA, scoped tokens, no publish creds in CI, signed commits/tags, branch
protection, an `cargo dist`/`cargo release` checklist, etc.), the timeline, and credit. Public, in
`docs/postmortems/`.

---

## Hardening checklist (do these *now*, before an incident)

- [ ] All maintainers: 2FA on GitHub **and** crates.io, ideally with a hardware security key (and
  recovery codes stored offline).
- [ ] crates.io publish tokens: **scoped** (to `spiffe-id`, publish-only), one per maintainer, not
  shared, not in CI. Releases are done locally from a clean checkout, not by a workflow.
- [ ] `main` branch protection: require PR + ≥1 approving review, required status checks (the CI
  jobs), no force-push, no branch deletion, no admin bypass, restrict who can push.
- [ ] Consider **required signed commits/tags** (`git config commit.gpgsign true`; protect `main`
  with "require signed commits") and signing release tags.
- [ ] `cargo install cargo-audit` and run `cargo audit` in CI (so a *future* dependency, or this
  crate appearing in RustSec, is flagged) — even though the crate is zero-dep today.
- [ ] A `CODEOWNERS` file so PRs go to the right reviewers.
- [ ] `dependabot.yml` for the GitHub Actions versions (the only "dependencies" this repo has).
- [ ] A release checklist (in `CONTRIBUTING.md` / `RELEASING.md`): clean checkout, full gate green,
  `cargo publish --dry-run`, tag, publish, verify on crates.io + docs.rs.
- [ ] Know how to reach GitHub Support and `help@crates.io` *before* you need to.
