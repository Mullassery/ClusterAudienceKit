# ClusterAudienceKit Security Audit

**Last Audited:** August 2026 (post remediation pass)
**Status:** One real SQL-injection vector found and fixed. Dependency
pinning and PII-handling status corrected below — the previous version of
this file predated an 2026-08-07 commit and described dependency-pinning and
clustering-implementation status that no longer matches the codebase.

---

## Fixed this pass

### 1. SQL injection in `sql_export.rs`
**Severity:** HIGH
**Status:** FIXED

`SQLExporter::export_segment`/`export_all_segments` built SQL via `format!()`
with `table_name` and the four `ColumnMapping` fields
(`customer_id`/`recency_score`/`frequency_score`/`monetary_score`)
interpolated directly, unescaped, into the generated query
(`build_query`/`build_where_clause`). Since these values are caller-supplied
(they flow straight from the Python-facing `export_segment_sql`/
`export_all_segments_sql` functions' `table_name`/`customer_id`/
`recency_score`/`frequency_score`/`monetary_score` parameters), a caller
passing e.g. `table_name="customers; DROP TABLE users; --"` would have had
that string emitted verbatim into the returned SQL string.

**Fix:** added `validate_identifier()`, called on `table_name` and every
`ColumnMapping` field before any string is built. Identifiers must be
alphanumeric/underscore, optionally dot-qualified (to still allow
schema-qualified names like BigQuery's `project.dataset.customers`), and
must not start with a digit; anything else is rejected with a clear error
before any SQL is constructed. Covered by new tests:
`test_rejects_sql_injection_in_table_name`,
`test_rejects_sql_injection_in_column_mapping`,
`test_accepts_schema_qualified_table_name`,
`test_rejects_empty_table_name` in `src/engine/sql_export.rs`, plus the
pre-existing `test_sql_no_injection_vulnerability` in
`tests/test_sql_export.py` (which was already passing against the intended
segment-name allow-list before this fix — this fix closes the previously
unvalidated table-name/column-name path specifically).

Note: segment *names* (`"Champions"`, `"AtRisk"`, etc.) were already safe —
they're looked up against a fixed internal `HashMap` of known segment
definitions and rejected with an error if not found, so there was no
injection surface there. The vulnerable inputs were specifically
`table_name` and the four column-name overrides.

---

## PII handling — closed this pass

Previously there was no way to anonymize or add differential-privacy noise
to customer data anywhere in the Python-facing API — `engine::privacy`
(differential privacy: Laplace/Gaussian mechanisms with a privacy-budget
tracker; k-anonymity: group-size checking, row suppression, numeric
generalization, information-loss measurement) existed with real Rust tests
but was never wired to Python. It's now exposed as `PyPrivacyBudget`,
`add_laplace_noise`, `add_gaussian_noise`, `check_k_anonymity`,
`suppress_to_k_anonymous`, `generalize_numeric`, and
`calculate_information_loss`. This doesn't retroactively anonymize existing
call sites — it's opt-in tooling callers now have available — but it closes
the gap where the capability existed in Rust and was simply unreachable.

---

## Dependency pinning — current status

`Cargo.toml` uses caret-range version requirements (`"0.16"`, `"0.22"`,
etc.) for all dependencies, which is normal, idiomatic Rust practice — NOT
"0 pinned, 12 floating" as a previous version of this document claimed.
Reproducible builds come from `Cargo.lock`, which **is** committed to the
repository (tracked in git; a stale `.gitignore` entry that would have
excluded it — despite it already being tracked, which `.gitignore` doesn't
retroactively affect, but was misleading — has been removed this pass).
`cargo build`/`maturin build` will therefore resolve to the exact locked
versions unless someone explicitly runs `cargo update`.

Python-side (`pyproject.toml`) dependencies use range pins
(`pandas>=2.0,<3`, `numpy>=1.24,<2.1`, `pyarrow>=10.0`), which is
appropriate for a library (as opposed to an application, where exact pins
via a lockfile are more common).

**Residual recommendation:** run `cargo audit` / `pip-audit` periodically in
CI to catch known-vulnerable versions within the allowed ranges; this repo
does not currently have that wired into `.github/workflows/`.

---

## Input validation

Rust-side functions generally validate their own preconditions with clear
errors (e.g. `kmeans`/`kprototypes` reject `n_clusters == 0`, `n_clusters >
n`, empty data — see `src/engine/clustering.rs` tests). There is no
Pydantic-style request-object validation layer, but this is a library, not
a network service accepting untrusted request bodies — the main untrusted-
input surface that actually matters here is the SQL-export identifier path
fixed above, plus generic malformed-input handling that already returns
`PyResult` errors rather than panicking across the FFI boundary (a panic
crossing the Rust/Python boundary is UB in PyO3; every public function
returns `Result`/`PyResult` rather than using `.unwrap()` on caller-supplied
data — this was spot-checked across the newly-wired modules' Python bindings
in `src/python.rs`).

---

## Resource limits

No explicit `MAX_CUSTOMERS`/`MAX_CLUSTERS`/`MAX_FEATURES` caps exist. For a
library invoked in-process (not a network-facing service), the caller
already controls how much data they hand it, so a hardcoded cap would just
be an arbitrary annoyance rather than a real mitigation; if this package
grows a server/API mode in the future, resource limits belong there, not in
the core library.

---

## Code quality gates (not strictly "security," but load-bearing for it)

- `cargo fmt --check`: clean, repo-wide (was not, before this pass).
- `cargo clippy --workspace -- -D warnings`: 43 pre-existing findings
  remain, all in modules not reachable from the Python API (either
  explicitly deferred or not yet wired — see `docs/ROADMAP_HONEST.md` for
  the full breakdown). Down from 214 before this pass.
- `cargo test --lib`: 421+ passing, 0 failing.
- Python test suite (`pytest tests/`): 225 passing, 2 conditionally skipped
  (scikit-learn comparison benchmarks, skipped when sklearn isn't
  installed).

---

## Testing

```bash
cargo audit          # not currently run in CI; recommended addition
pip-audit --strict    # not currently run in CI; recommended addition
cargo clippy --workspace -- -D warnings
cargo fmt --check
cargo test --lib
pytest tests/
```

---

## Deployment notes

- This is a library, not a deployed service — there is no "run with
  non-root user" concern in the way a previous version of this document
  implied (no daemon, no server process ships in this package as of this
  release; `engine::dashboard`/MCP-connector documentation describing a
  running service were fictional and have been removed — see
  `docs/ROADMAP_HONEST.md`).
- If you build a service around this library, apply standard practice
  there (non-root user, resource limits, input validation at your API
  boundary) — those concerns belong in your service layer, not this core
  library.
