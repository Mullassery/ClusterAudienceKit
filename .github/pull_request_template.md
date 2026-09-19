## What does this change do, and why?

<!-- Link an issue if there is one. -->

## How was this tested?

<!--
This project's real test paths (see CONTRIBUTING.md):
- Rust: `cargo test --release` (works on Linux/CI; currently does not run
  standalone on macOS — see docs/ROADMAP_HONEST.md)
- Python, end-to-end through the compiled extension (works on macOS too):
  `maturin develop --release && pytest tests/`
-->

- [ ] Added/updated a Rust `#[cfg(test)]` unit test in the module you changed
- [ ] Added/updated a Python-level test in `tests/` exercising the real PyO3 binding
- [ ] `cargo fmt --all -- --check` passes
- [ ] `pytest tests/` passes

## Anything reviewers should pay extra attention to?
