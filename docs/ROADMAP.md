# Roadmap: ClusterAudienceKit

> Generated via competitive intelligence audit. Base don actual codebase state and market analysis. 

## Current State Assessment

- **Type:** Python/Rust hybrid (Rust core compiled as Python extension via `clusteraudiencekit` module)
- **Maturity:** Production-ready core with extensive functionality (RFM, clustering, churn, LTV, SQL export for 8 dialects, differential privacy, streaming, drift detection, lookalikes, cohorts, forecasting, price elasticity, etc.)
- **Strengths:**
  - High-performance Rust core exposed to Python
  - Comprehensive segmentation capabilities (13 RFM segments)
  - Enterprise features: differential privacy, k-anonymity, multi-dialect SQL export
  - Real-time streaming segmentation and drift detection
  - Extensive cohort/lifecycle/trend/forecasting capabilities
  - Squid CRM integration schema
- **Current Gaps:** API documentation completeness, broader language bindings, cloud deployment patterns, UI/visualization layer, performance benchmarks published

## Competitive Landscape Analysis

**Key competitors/alternatives:**
- **scikit-learn + pandas** (open source, Python-native, wide adoption) - weaker in privacy/SQL export/real-time
- **RFM segmentation tools** (e.g., Optimove, Blueshift, Amperity) - commercial, enterprise-focused
- **Customer Data Platforms** (Segment, mParticle, Rudderstack) - broader CDP focus, less analytical depth
- **Clustering libraries** (hdbscan, kmodes) - focused solely on clustering
- **Amplitude/ Mixpanel cohorts** - product analytics focused

**Competitive Positioning:** "Privacy-first, warehouse-native customer segmentation engine" - differentiate via Rust performance, built-in DP/k-anonymity, direct SQL export to 8 dialects, real-time drift detection. Best-in-class for privacy-constrained analytics teams.

## Critical Gaps to Fill

1. **API Documentation** - Need comprehensive API reference (Sphinx/mkdocs), examples for each module
2. **Performance Benchmarks** - Publish benchmarks vs scikit-learn for scale validation
3. **Integration Examples** - dbt/BigQuery/Snowflake integration patterns
4. **Testing in CI** - Ensure full test suite runs, add mutation testing
5. **Packaging** - Wheels for common platforms, better installation docs

## Roadmap by Timeline

### Short-term (0-30 days) - Quick Wins & Critical Fixes

| Item | Priority | Effort | Impact | Description |
|---|---|---|---|---|
| API docs site | P0 | M | High | Generate comprehensive API docs from docstrings, publish to GitHub Pages |
| Quickstart examples | P0 | S | High | Add 5-10 end-to-end examples (RFM, clustering, churn, export) |
| Benchmark suite | P0 | M | High | Create reproducible benchmarks vs sklearn, publish results |
| CI coverage reporting | P0 | S | High | Add coverage badges, ensure tests pass in CI |
| Type stubs improvements | P1 | S | High | Complete .pyi stubs for all public APIs |

### Medium-term (30-90 days) - Core Features & Integrations

| Item | Priority | Effort | Impact | Description |
|---|---|---|---|---|
| dbt integration | P0 | M | High | Package as dbt metrics/transform, or dbt package for segment materialization |
| Polars support | P0 | M | High | Add Polars DataFrame support alongside pandas for better perf |
| Visualization utilities | P1 | M | High | Add plotting helpers (segment charts, elbow plots, etc.) |
| Model registry/serialization | P1 | M | High | Save/load segmenters, cohorts with versioning |
| Spark integration | P1 | L | High | Add PySpark DataFrame support for big data |

### Long-term (90+ days) - Strategic Growth

| Item | Priority | Effort | Impact | Description |
|---|---|---|---|---|
| Cloud-native API service | P2 | L | High | FastAPI service wrapper for remote segmentation |
| Feature store integrations | P2 | L | High | Feast/Tecton integration for features |
| ML pipeline integration | P2 | L | High | MLflow integration for experiment tracking |
| Real-time streaming connectors | P2 | L | High | Kafka/Kinesis connectors for streaming engine |
| Enterprise SSO/RBAC | P2 | L | Med-High | If deployed as service |

## Prioritized Feature List with Effort/Impact

| Feature | Category | Effort | Impact (1-10) | Priority | Rationale |
|---|---|---|---|---|---|
| API documentation | DX | S | 10 | P0 | Adoption blocker |
| Polars support | Performance | M | 9 | P0 | Modern data stack requirement |
| dbt package | Integration | M | 9 | P0 | Warehouse-native workflow |
| Benchmarks published | Marketing/Perf | S | 8 | P0 | Differentiation proof |
| Spark support | Scale | L | 9 | P1 | Enterprise scale |
| Visualization helpers | UX | M | 7 | P1 | Easier adoption |
| FastAPI service | Deployment | L | 8 | P2 | Service deployment option |

## Success Metrics

| Metric | Baseline | 30-day | 90-day | Measurement |
|---|---|---|---|---|
| API docs coverage | Partial | 100% public APIs | Complete with examples | Docs review |
| Test coverage | Unknown | >80% | >90% | pytest-cov |
| PyPI downloads | Baseline | +50% | +150% | PyPI stats |
| GitHub stars | Baseline | +25% | +100% | GitHub |
| Integration examples | Few | 10 examples | 20+ examples | Count in repo |

## Technical Debt to Address

| Debt Item | Severity | Effort | Plan |
|---|---|---|---|
| Type stub completeness | High | S | Complete all public APIs in .pyi files |
| Error handling consistency | High | M | Standardize exceptions, improve messages |
| Memory efficiency for large datasets | High | M | Streaming/chunking for memory-intensive ops |
| Test organization | Med-High | S | Better test structure, fixtures |
| Dependency management | Med | S | Pin versions, security updates |

## Next Steps

1. **Week 1:** Create comprehensive API docs structure, add missing docstrings
2. **Week 2:** Build benchmark suite comparing to sklearn baselines
3. **Week 3:** Add Polars support to core functions
4. **Week 4:** Publish docs and announce improvements
