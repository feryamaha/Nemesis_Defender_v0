# Contributing to Nemesis Defender

Thank you for your interest in contributing to Nemesis Defender! This is a project focused on deterministic security and governance for AI-assisted development. To maintain the robustness of the software, we follow strict development guidelines.

## Development Environment Setup

Nemesis is written primarily in Rust. To get started, you will need:

* Rust (minimum v1.70) and Cargo
* Clang/LLVM (required to compile the core)
* Linux environment (if you want to modify or test the eBPF/LSM Layer 3)

### Cloning and Building the Project

To clone and build, run the normal git clone commands and then:

```bash
cargo build --release --workspace
```

## How to Run the Tests

No code change will be accepted if it breaks the existing test suite. Before opening a Pull Request, run:

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

## Code Standards

* **Security first:** `unsafe` code must be avoided as much as possible and requires explicit justification documented in a comment.
* **IDE-agnostic:** any new logic added to the `nemesis-defender` library must remain platform- and code-editor-agnostic.
* **Regression tests:** if you fixed a bug or a bypass, include a synthetic test case that covers that scenario to prevent future regressions.

## Pull Request (PR) Process

1. Fork the repository and create your branch from `main`.
2. Ensure all tests pass locally.
3. Make sure your change is documented.
4. Open the PR with a clear description of the problem you are solving and the impact on the framework's security/performance.

## Reporting Security Vulnerabilities

**Important notice:** if your contribution is the discovery of a critical security vulnerability or an exploitable bypass, **do not open a public PR**. Follow the process described in the [SECURITY.md](SECURITY.md) file.

## License of Contributions (DCO)

By submitting a contribution to this project, you agree that:

1. Your contribution will be licensed under the same license as the project (**GNU AGPL v3.0**).
2. You certify the origin of the code you are submitting, in accordance with the **Developer Certificate of Origin (DCO)** — that is, you declare you have the right to submit this code under the project's license.
3. You grant the author/maintainer the right to also license your contribution under a **separate commercial license** (dual licensing). This is necessary because the project retains the commercial licensing option, and contributions under pure AGPL would prevent the maintainer from relicensing. By contributing, you agree to this grant.

To certify, add the line `Signed-off-by: Your Name <your@email.com>` at the end of each commit (use `git commit -s` to do this automatically).

This keeps the codebase legally clean and ensures that every contribution can be integrated and maintained without ambiguity of rights.