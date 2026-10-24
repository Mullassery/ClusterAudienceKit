# Audit findings — 2026-10-19

This file records the repository-specific findings from the portfolio audit
conducted on 2026-10-19. It is documentation only; no code or release artifacts
were changed in this pass unless explicitly noted under "Implementation status".

Sources: the repository's own README/source/workflows, GitHub metadata and
Actions runs, PyPI metadata, and official competitor documentation. Claims are
marked as verified, inferred, or open questions. "Absent" means not found in the
inspected repository/docs, not proof that it cannot exist.

## What it is

Rust-powered customer segmentation with a Python API: RFM, K-Means/K-Prototypes, churn, CLV, lookalikes, cohorts, privacy primitives, streaming segmentation, and SQL export.

## Verified strengths

- Broad, real feature set (RFM, clustering, churn AUC, CLV, SQL export for 8 dialects, DP/k-anonymity, streaming, drift, cohorts, lifecycle).
- Rust core with Python bindings; Apache-2.0; tests for clustering, performance, SQL export, and wired modules.
- Clear MarTech positioning and cross-links to PyReverseETL/MessageBirds.

## Verified gaps and user-facing risks

- PyPI publishes only a macOS arm64 wheel; Linux/Windows fall back to a Rust build.
- Latest GitHub release is v1.0.0 (2026-07-07) while PyPI is 7.4.2 — release history is not trustworthy.
- No live warehouse/data connectors; no statistical validation against lifetimes/PyMC-Marketing; no committed benchmark vs scikit-learn.
- Version number (7.x) implies maturity the project does not claim elsewhere.

## Direct competitors

- scikit-learn (general ML), lifetimes (BG/NBD + Gamma-Gamma CLV), PyMC-Marketing (Bayesian CLV), Segment Personas/Adobe RTCDP (commercial).
- The library's wedge is an integrated MarTech workflow, not better clustering primitives.

## Ranked next actions

- P1: publish Linux/Windows wheels via maturin + Trusted Publishing (M).
- P1: align GitHub releases/tags with PyPI (S).
- P2: validate CLV/churn against lifetimes/PyMC on a public dataset (M).
- P2: warehouse-native segment export (Snowflake/BigQuery) (M).

## Do not build

- A full CDP or warehouse. Integrate with MessageBirds/PyReverseETL instead.

## Evidence gaps / validation needed

- No campaign/segment outcome data; no CLV accuracy benchmark; no live warehouse test.

## Recommended disposition

Invest as a MarTech component, but only after packaging and release hygiene.

## Implementation status

No code changes in this pass.
