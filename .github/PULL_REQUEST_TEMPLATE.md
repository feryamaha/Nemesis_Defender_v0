## Objective
[1-4 lines: what was done and why. Reference the spec/plan.]

## Type of change
- [ ] Bug fix
- [ ] New feature
- [ ] Regression test (covering an already-fixed bug/bypass)
- [ ] Documentation
- [ ] Refactor / performance

## Affected Files
- `path/to/file.rs` [new|modified]
- `path/to/other/file.rs` [modified]

## Implementations Made

### File: `path/to/file.rs` (new|modified)
- [What was created/modified in detail. Technical decision. Rust pattern followed.]

### File: `path/to/other/file.rs` (modified)
- [What was modified. Technical decision.]

## Acceptance Criteria
- [x] `cargo check --workspace`: PASS
- [x] `cargo test --workspace`: PASS
- [x] No Nemesis violations
- [x] Idiomatic Rust code

## Benefits
[Reuse, decoupling, security, performance, enforceability]

## Additional Notes
[Additional context if relevant]

> ⚠️ If this PR is related to a **security flaw or exploitable bypass**, stop. Do not describe the exploit here — follow the [SECURITY.md](SECURITY.md) and report it privately first.

## CLI Table

| Command | Result (OK/FAIL) | Observations |
|---------|------------------|--------------|
| `cargo check --workspace` | OK | Valid compilation |
| `cargo test --workspace` | OK | Tests pass |
| Static pentest | OK | 224/224 PASS (100%) |
| Full live pentest | OK | 74/74, 0 gaps, SELF-SUFFICIENT |

## DCO
By opening this PR, I confirm that my contribution will be licensed under the **GNU AGPL v3.0** (and I grant the maintainer the right to dual licensing, as per the CONTRIBUTING.md) and I certify the origin of the code in accordance with the **Developer Certificate of Origin**.