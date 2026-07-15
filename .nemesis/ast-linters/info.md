# AST Linters — Nemesis

## Objective

Isolated crate for semantic validation of source code via tree-sitter.
Complements the regex validations of the `workflow_enforcer` (Class A) with analysis
based on the syntax tree (Class B of the audit).

## Current Status

- [x] Crate structure created
- [x] `language.rs` — language detection by extension
- [x] `parser.rs` — tree-sitter wrapper (TS/JS)
- [x] Declarative rules system via `rules.toml` (build.rs generates visitors)
- [x] Manual visitors for complex cases (exhaustive_deps, no_floating_promises)
- [x] 20 declarative rules in `rules.toml` (TS/JS, React, Python, Go, Java, Rust)
- [x] 30+ manual visitors for specific cases (React hooks, security, typing)
- [x] Critical Severity added to the enum for blocking
- [x] `any_via_alias.rs` — detects `type X = any`
- [x] `conditional_hooks.rs` — detects hooks inside if/for/while
- [x] `fetch_in_component.rs` — detects fetch/axios in a component
- [x] `exhaustive_deps.rs` — detects useEffect with incomplete deps
- [x] `no_floating_promises.rs` — detects promises without await
- [x] `unused_vars.rs` — detects declared but unused variables
- [x] `cache.rs` — LRU cache for parsed AST
- [x] `validator.rs` — `validate_semantic()` integrated into the workflow_enforcer
- [x] Clean compilation + 55 tests passing (lib + generated_rules)
- [x] Integrated into `workflow_enforcer.rs` (library) and `pretool-hook.rs` (CLI)
- [x] Static pentest: 200/203 tests passing (98.5%) — criterion ≥98% met
- [x] PHASE 1-4: 17 new rules added (BUILD-BREAK, React, Typing)
- [x] PHASE 5A: Static validation against the binary completed
- [ ] Step 5B: Real pentest (LLM model writing code)
- [ ] Step 5C: Integration into cargo test/check

### Known Gaps (accepted)
- **T-8.13 (no-obj-calls)**: `const obj={};obj()` — edge case, low priority
- **T-8.14 (sparse arrays)**: `const arr=[1,,3]` — edge case, low priority
- **T-8.21 (unsafe assignment)**: `const x:string=123` — **tsc's responsibility**, requires type inference (outside the scope of ast-linters)

### Expected False Positives (CORRECT)
- **T-26.01**: docs/aws-guide.md — legitimate documentation must not block ✅
- **T-26.06**: docs/security-guide.md — legitimate security guide must not block ✅

## Dependencies

```toml
tree-sitter = "0.24"
tree-sitter-typescript = "0.23"
tree-sitter-javascript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-go = "0.23"
lru = "0.12"
```

## How to Test

```bash
cargo build -p ast-linters
cargo test -p ast-linters
```

## Architectural Decisions

1. **Separate crate**: The tree-sitter dependencies stay isolated in `ast-linters`,
   without polluting the main `nemesis` crate. Communication with the `workflow_enforcer`
   happens via the public function `validate_semantic()`.

2. **Never break the hook**: If the parse fails (unsupported language, malformed
   file), `validate_semantic()` returns an empty list. Failures are logged
   only at debug level. The hook never fails to block because of the AST.

3. **LRU cache**: Avoids re-parsing the same file during the same session.
   The key is `(path, content_hash)`. 32 entries. Automatic invalidation.

4. **Language by extension**: Simple and direct mapping. New languages
   require: (1) adding a parser in Cargo.toml, (2) adding the extension in
   `language.rs`, (3) implementing specific visitors.

## Structure

```
src/
├── lib.rs                  # Public re-exports
├── language.rs             # Language enum + detection by extension
├── parser.rs               # tree-sitter wrapper
├── cache.rs                # LRU cache
├── validator.rs            # validate_semantic() → Vec<Violation>
└── visitors/
    ├── mod.rs              # Re-exports of the visitors
    ├── any_via_alias.rs
    ├── conditional_hooks.rs
    ├── fetch_in_component.rs
    ├── exhaustive_deps.rs
    └── unused_vars.rs
```

## Next Steps

- Adapt validations for each language
- Step 5: docs + logging with layer
