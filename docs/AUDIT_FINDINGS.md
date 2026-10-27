# Audit Report: ClusterAudienceKit

## Overview
- **Type**: Rust-powered Python extension (PyO3/maturin) for customer segmentation/marketing analytics
- **Language**: Rust (cdylib + rlib) + Python bindings
- **Version**: 7.5.2 (Cargo.toml/pyproject)
- **License**: Apache-2.0
- **Status**: Production/Stable (PyPI `clusteraudiencekit`)
- **Purpose**: RFM, clustering (KMeans/K-Prototypes), churn/CLV, cohort/lifecycle, lookalike, drift, streaming, privacy (DP/k-anonymity), SQL export (8 dialects), revenue/price intelligence, neural net regressor, Squid CRM schema contract

## Architecture
- **Core**: Rust engine in `src/engine/` with 30+ modules (rfm, clustering, k_estimation, quality_metrics, churn_prediction, clv, privacy, streaming, drift_detection, lookalike, cohorts, lifecycle, behavioral, temporal_analytics, profiling, segment_intelligence, revenue_intelligence, price_intelligence, neural_networks, pattern_discovery, governance, b2b_*, activation*, dashboard, plugins, algorithms, segments, sql_export)
- **Bindings**: `src/python.rs` (PyO3 classes/functions), `src/lib.rs` exposes module, `clusteraudiencekit/` contains Python package layer (`__init__.py`, `squid.py`, etc.)
- **Utils**: conversions/validation
- **Data/Contracts**: `schemas/squid/v1/` for Squid CRM segment/membership JSON schemas

## Entry Points
- Python: `import clusteraudiencekit` via compiled extension; high-level functions/classes listed in README (calculate_rfm, kmeans, ChurnPrediction, CustomerLTV, export_segment_sql, PyPrivacyBudget, PyStreamingSegmentationEngine, etc.)
- Rust lib: `src/lib.rs` registers PyO3 module

## Dependencies
**Rust**: ndarray 0.16/ndarray-stats 0.7, pyo3 0.29 (abi3-py39, extension-module default), numpy 0.29, arrow 55.0, parquet 59.0, rayon 1.10, chrono 0.4, serde 1.0, rand 0.8, thiserror 2.0, log 0.4; dev: criterion 0.8, proptest 1.4
**Python**: pandas>=2.0,<3, numpy>=1.24,<2.3, pyarrow>=10.0; optional preprocessing (scikit-learn), dev/docs

## Module Structure
- Large monorepo-like Rust engine with comprehensive feature set across marketing analytics domains
- Python package mirrors/extends with Squid integration helpers
- Schemas present for CRM contract

## Data/State/APIs/Contracts
- In-memory computations over numpy arrays; Arrow/Parquet deps present (possibly for I/O/export paths)
- SQL generation with dialect support (8) and claims of injection-safety
- Privacy primitives (Laplace/Gaussian noise, k-anonymity)
- Squid v1 schema contract defined

## Security Observations
- SQL export claims injection-safe (worth verifying implementation)
- Privacy features present (DP/k-anonymity) - good for regulated use
- Dependencies include arrow/parquet (file format parsing - general surface)
- No obvious secret handling patterns visible from README; extension-module links without bundling libpython (documented tradeoff)

## Tests/CI
- pytest markers defined (benchmark), dev deps include pytest/pytest-benchmark
- GitHub Actions badge in README (`tests.yml`) - CI present
- Tests likely under `tests/` (directory not shown in listing but common)

## Findings

| Severity | Finding | Evidence | Impact | Root Cause Hypothesis | Recommendation |
|----------|---------|----------|--------|----------------------|---------------|
| Medium | Very large surface area (many engines/features) in single crate | `src/engine/*.rs` (30+ files), README lists 30+ capabilities | Maintenance complexity, harder to audit/test comprehensively | Ambitious scope covering full MarTech stack | Consider feature flags/modules split, ensure each major engine has tests/docs |
| Medium | Arrow/Parquet deps present but usage scope unclear from top-level | Cargo.toml deps | Larger binary/wheel size, parsing attack surface if used with untrusted files | Data export/import support | Verify I/O paths are hardened, validate schemas |
| Low | NumPy API version pinning tight (<2.3) | pyproject deps | Compatibility constraints | Stability for extension | Document supported ranges |
| Low | abi3-py39 limits Python API usage patterns | Cargo/pyproject | Can't use very new CPython APIs | Wheel compatibility | Acceptable tradeoff |

## Strengths
- Comprehensive, real implementations across segmentation lifecycle
- Rust performance + PyO3 bindings with abi3 wheels strategy
- Strong privacy/governance features
- Clear SQL dialect support with safety claims
- Well-documented API surface

## Limitations
- Large scope increases attack/complexity surface
- Some advanced features (neural net) add complexity
- Need to verify "injection-safe SQL" and privacy guarantees with tests

## Prioritized Recommendations
1. **High**: Audit SQL generation for injection safety with tests
2. **High**: Review privacy implementations (DP noise, k-anonymity) for correctness
3. **Medium**: Ensure CI runs tests across Python versions, check coverage
4. **Medium**: Document security model (SSRF/file I/O/network) if any
5. **Low**: Consider module boundaries/feature flags for optional heavy deps
