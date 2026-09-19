# Archive

Stale, superseded, or fabricated docs, kept for history rather than deleted
outright. None of these are linked from `README.md` or any current doc, and
none should be treated as accurate about the current codebase (v7.3.0).
`git log --follow <path>` on any file here shows its original history.

## Fabricated or aspirational content (never matched the shipped code)

These documented an `AudienceSegmenter(method="rfm_kmeans", n_clusters=4, ...)`
constructor, a `.fit_predict(df)` method, a `.update()` / `.segment_stability()`
streaming API, and specific performance numbers ("2.7 hours at 100k customers",
"10-25x faster", "20x faster") — none of which exist or were ever measured
against this codebase. The real constructor is `AudienceSegmenter(n_clusters,
n_jobs=None)` taking a numeric feature matrix (see `README.md`). A prior pass
added an honesty banner to these five instead of rewriting them; this pass
moved them out of `docs/` entirely since a banner isn't enough to stop them
being read as reference material:

- `api-reference.md` — documents the fictional constructor/method signatures above.
- `getting-started-simple.md` — walks through the same fictional API.
- `comparison.md` — feature-comparison table and "13 lines vs 4 lines" pitch
  built on the fictional API and unverified benchmark numbers.
- `performance-comparison.md` — a full fabricated benchmark table
  ("10-25x faster", "SIMD", "<200ms at 1M customers") with no corresponding
  benchmark code for those exact claims (the real benchmarks are in
  `benches/benchmarks.rs`, and only cover `kmeans`/`calculate_rfm` at 10k/100k
  rows on synthetic data, on the author's machine — see
  `docs/ROADMAP_HONEST.md`).
- `press-release.md` — draft social-media/press-release copy for multiple
  platforms, "MIT licensed" (repo is Apache-2.0), "Phase 1 implementation in
  progress" language, same fabricated benchmark numbers.
- `troubleshooting.md` — built around the same fictional `fit(df)`/
  DataFrame-column-based API and links to `api-reference.md` above.
- `error-catalog.json` — an unused data file (nothing in `src/` or the Python
  package loads it) of error patterns for the same fictional DataFrame-based
  `fit()` API.
- `WORKFLOW_INTEGRATION.md` — documents a CLI (`clusteraudiencekit
  create-audience`, `refresh-audience`, ...) and a REST server
  (`python -m clusteraudiencekit.server` on port 8002). Neither exists:
  `pyproject.toml` defines no `[project.scripts]` entry point, and there is no
  server module anywhere in the package.
- `ARCHITECTURE.md.fabricated-ecosystem` (was root `ARCHITECTURE.md`) —
  described integration with four other products (`StatGuardian`,
  `PyCustomerJourney`, `PyReverseETL`, `PyStreamMCP`) via Rust/Python code
  samples (`use pystreammcp::Discovery;`, `use statguardian::ValidationGate;`)
  that reference crates/packages this project has no dependency on anywhere
  in `Cargo.toml` or `pyproject.toml`, and described a `core/src/` module
  layout (`audience.rs`, `segment.rs`, `clustering/`, `scoring/`, `storage/`)
  that has never existed in this repo (the real layout is a flat `src/engine/`
  with 30+ modules — see `docs/architecture/README.md` for the real one).
- `architecture.md.stale` (was `docs/architecture.md`) — a second,
  independently-stale architecture doc describing yet another fictional
  module layout (`segmentation/`, `metrics/`, `profiling/`, `drift/`, `io/`).

## Superseded by a current, canonical doc

- `ROADMAP.md` — the old roadmap (last said "v2.0.0", described a CLI/REST
  "Workflow Integration" milestone as already shipped — see
  `WORKFLOW_INTEGRATION.md` above for why that's false). Superseded by
  `docs/ROADMAP_HONEST.md`, which is the maintained, accurate roadmap.
- `CONTRIBUTING.md.claude-dupe` (was `docs/CONTRIBUTING.md`) — despite the
  filename, its content was a `CLAUDE.md`-style AI-assistant briefing
  describing a fictional `core/src/` crate layout, not contributor
  instructions. The real `CONTRIBUTING.md` lives at the repo root.
- `SECURITY.md.stale-duplicate` (was `docs/SECURITY.md`) — an older draft
  referencing a `PRODUCTION_AUDIT_REPORT.md` that doesn't exist anywhere in
  this repo, labeled version `0.1.0` ("NO PRODUCTION USE - beta"), which
  doesn't match the current `7.3.0` / `Production/Stable` classifier. The
  real `SECURITY.md` lives at the repo root.
- `PYPI_UPLOAD.md` — release instructions for `v1.5.0` using `python3 -m
  build --wheel`; the project is now on `7.3.0` and uses `maturin`. Nobody
  has replaced this with current instructions yet — see the "pending" bucket
  in `docs/ROADMAP_HONEST.md`.
- `CI_ERRORS.md` (was `.github/CI_ERRORS.md`) — a generic scaffold from some
  automated org-wide CI-scanning tool, never filled in for this repo: it links
  to `../../CI_ERRORS_REPORT.md` and `.github/TROUBLESHOOTING.md`, neither of
  which exists anywhere, and contains unsubstituted template variables
  (`$REPO_NAME`, `$repo`).
