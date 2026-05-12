# Playbooks — `spiffe-id`

Operational runbooks for the maintainers of `sanguinehost/wimse` (the `spiffe-id` crate). These are
checklists for the bad days. Keep them current; rehearse the hardening bits *before* you need the
incident bits.

| Playbook | Use it when… |
|---|---|
| **[`vulnerability-response.md`](vulnerability-response.md)** | A security report arrives (via `security@twn.systems` or GitHub private vulnerability reporting), or you realise a bug *might* be a vulnerability. Covers: triage & CVSS scoring → fixing privately → drafting the GHSA & requesting a CVE → coordinating disclosure → the release/disclosure order → the RustSec advisory → `cargo yank` → the post-mortem. **This is the GHSA playbook.** |
| **[`maintainer-compromise.md`](maintainer-compromise.md)** | You suspect a maintainer's GitHub account, a crates.io publish token, a signing key, or the CI pipeline has been (or might have been) compromised — i.e. someone could have *pushed* or *published* something malicious. Bias-to-action containment, blast-radius assessment, recovery, notifications, post-mortem — plus a **hardening checklist** to do now, before any incident. If a malicious version actually shipped, you also run the advisory half of `vulnerability-response.md`. |

Both playbooks reference each other where they overlap; the threat surface they exist to cover is
laid out (for the maintainer audience) in [`../THREAT-MODEL.md`](../THREAT-MODEL.md#audience-3--the-maintainers--the-supply-chain),
and the public-facing reporting process is [`../../SECURITY.md`](../../SECURITY.md).

A general rule for both: **if you're not sure whether to run the playbook, run it.** Containment and
acknowledgement are cheap and reversible; a poisoned release in the wild, or a security report that
sat unread, are not.

## Future playbooks (add as needed)

- **Releasing** — currently a checklist in [`../../CONTRIBUTING.md`](../../CONTRIBUTING.md#releasing-maintainers);
  promote it to its own `releasing.md` if it grows.
- **Yanking / un-yanking a crates.io version** — the procedure and the criteria; currently covered
  inline in the two playbooks above.
- **Deprecating / archiving the crate** — if it's ever superseded; the governance side is in
  [`../../GOVERNANCE.md`](../../GOVERNANCE.md).

When you write one: same shape as the existing two — *Audience*, *When to use*, numbered steps with
checkboxes, a quick-reference at the end, and a hardening section if there are "do these now" items.
