# wimse

**SPIFFE/WIMSE URI parsing library. Zero-dep, no_std-friendly. Consumed by AIM and agent-identity-resolver.**

## Status

🚧 **Placeholder.** Reserved for the upcoming extraction from
[`sanguinehost/ferrousgate`](https://github.com/sanguinehost/ferrousgate).

| Field | Value |
|---|---|
| Source crate | `(new — not yet implemented)` (in ferrousgate) |
| Source iteration | iter-18 (deferred — see iter-13 spec § Deferred) |
| License | MIT (matches source workspace) |

## Why move it here

Sanguinehost owns the AI / agent-harness substrate; ferrousgate stays the
MCP gateway service. Crates that any sanguinehost-owned AI workload would
want to consume (memory, identity, classify, embeddings, indexing,
context-routing, middlewares) live in their own repos here so they can
be versioned and pinned independently.

## Extraction plan

Mechanical, once iter-13 / iter-13b land on ferrousgate. For
non-identity-touching crates this can run any time.

```bash
# in a clean clone of sanguinehost/ferrousgate:
git filter-repo --subdirectory-filter (new — not yet implemented) --tag-rename ':wimse-pre-extract-'
git remote add origin git@github.com:sanguinehost/wimse.git
git push origin main
```

## Related repos

- [`sanguinehost/ferrousgate`](https://github.com/sanguinehost/ferrousgate) — current home; will switch to git-dep on this repo post-extraction
- [`sanguinehost/aim`](https://github.com/sanguinehost/aim) — agent IdP
- [`sanguinehost/agent-identity-resolver`](https://github.com/sanguinehost/agent-identity-resolver), [`agent-classify`](https://github.com/sanguinehost/agent-classify), [`agent-memory`](https://github.com/sanguinehost/agent-memory), [`authzed-rs`](https://github.com/sanguinehost/authzed-rs) — sibling extractions
