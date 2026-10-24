# Competitive intelligence in roadmap and upgrade work

**Policy date:** 2026-10-19

Competitive findings are an input to product decisions, not a one-off report.
Every pending roadmap item, new feature, and routine dependency upgrade should
check the repository's [`AUDIT_FINDINGS.md`](AUDIT_FINDINGS.md) before work
starts, and record the decision in the PR or changelog.

## Required check before building

1. **Read `AUDIT_FINDINGS.md`.** Find the relevant competitor row and the
   ranked next actions. If the finding is stale, update it with a dated source.
2. **Name the competitor and the user problem.** A feature is not justified by
   "competitor has it". State the workflow it unlocks, who needs it, and how
   often. Mark unverified demand as a hypothesis.
3. **Prefer reuse over new code.** Check whether an existing Mullassery repo
   already provides the capability (see `BUNDLES.md` in the profile repo and
   the bundle role file in this repo, when present). Typical reuse:
   - crawling/fetching/robots/SSRF → **MudFish**
   - architecture maps / PR diffs → **CodeAtlas**
   - data-quality contracts / validation → **StatGuardian**
   - token counting / cost tables → **PyTokenCalc**
   - dependency graph / advisories / SBOM → **PyDeepMesh**
   - segmentation / CLV / churn → **ClusterAudienceKit**
   - activation / lineage / PII → **PyReverseETL**
   - fleet state sync → **SyncFleet** (only after conflict resolution)
4. **Choose one disposition:** build, integrate, defer, or reject. Write it
   down. A rejection with a reason is a successful outcome.
5. **For routine upgrades**, check whether the upgrade closes a known
   competitor gap or breaks a differentiated capability. Run the tests that
   cover the affected path; do not merge a green build that silently changes a
   user-visible contract.

## PR / changelog checklist

- [ ] Competitor and source URL/date recorded (or "no direct competitor").
- [ ] User impact and frequency stated; unverified demand labelled.
- [ ] Reuse check done against sibling repos/bundles.
- [ ] Failure mode and rollback considered (especially auth, PII, cost).
- [ ] Docs/roadmap status updated honestly.
- [ ] Tests or a manual verification command included.

## Review cadence

- Refresh the competitor row when a release changes the capability.
- Re-check the full `AUDIT_FINDINGS.md` at least quarterly and before a major
  release.
- If a finding is wrong or stale, correct it in the same PR that relies on it.

## Non-goals

- Copying features for parity.
- Adding a dependency just because a competitor uses it.
- Treating star counts or download counts as proof of user need.

## OSS gap analysis — 2026-10-24

Scope: Compared ClusterAudienceKit's implemented RFM/clustering/CLV surface against the two libraries users actually substitute for its numeric work — scikit-learn (clustering/preprocessing) and PyMC-Marketing (probabilistic CLV) — plus the archived-but-canonical lifetimes package. Assessed only features verified in this repo's `python.rs` surface or those projects' own docs/releases.

### Comparable OSS projects
| Project | GitHub | License | Last activity | Why it is the comparison |
| --- | --- | --- | --- | --- |
| scikit-learn | https://github.com/scikit-learn/scikit-learn | BSD-3-Clause | 1.9.1, 2026-09-11; pushed 2026-10-23 | Default choice for KMeans, preprocessing, metrics and model selection; the README benchmarks against it directly |
| PyMC-Marketing | https://github.com/pymc-labs/pymc-marketing | Apache-2.0 | 1.3.0, 2026-10-22 | Actively maintained Bayesian CLV/BTYD and MMM toolbox; the modern CLV alternative the README names as open validation work |
| lifetimes | https://github.com/CamDavidsonPilon/lifetimes | MIT | Archived; v0.11.3 (2020-07-06), last push 2024-06-28 | Canonical BG/NBD + Gamma-Gamma implementation and the package this repo's simple CLV is compared with |

PyMC-Marketing and scikit-learn cover the 2024-2026 activity requirement. lifetimes is included because it remains the reference implementation for the CLV comparison, but it is archived and should not be treated as a maintained dependency.

### Capability comparison
| Capability | Ours | scikit-learn | PyMC-Marketing | lifetimes | Gap |
| --- | --- | --- | --- | --- | --- |
| RFM scoring | Yes | No | No | No | None |
| KMeans clustering | Yes | Yes | No | No | Parity on core algorithm |
| Automatic K selection | Yes | Partial | No | No | Ours broader (elbow/gap/silhouette/combined) |
| Internal feature scaling | No | Yes | Partial | No | We lack it |
| Mixed-type (K-Prototypes) clustering | Partial | No | No | No | Python entry point is numeric-only |
| Churn scoring | Yes | No | Partial | Partial | Ours has explicit scorer; theirs proxy via retention |
| Probabilistic CLV (BG/NBD, Gamma-Gamma) | No | No | Yes | Yes | We have a weaker deterministic formula |
| Uncertainty intervals on CLV | No | No | Yes | Partial | We lack it |
| Privacy (DP, k-anonymity) | Yes | No | No | No | Ours only |
| SQL export to warehouses | Yes | No | No | No | Ours only |
| Streaming/online segmentation | Yes | No | No | No | Ours only |
| Model diagnostics / backtesting | Partial | Yes | Yes | Partial | We lack rigorous validation |
| Multi-platform wheels | Partial | Yes | Yes | Yes | macOS ARM64 only on PyPI |

The "Partial" for our mixed-type clustering reflects `AudienceSegmenter.fit()` accepting a numeric matrix only; mixed-data K-Prototypes exists in Rust but is not reachable from that Python class. Our "Partial" diagnostics means drift and quality metrics exist, but there is no CLV/churn holdout benchmark against lifetimes or PyMC-Marketing. PyMC-Marketing's CLV output is posterior-based, so uncertainty comes by construction rather than as a separate feature.

### Gaps that matter (ranked)
1. **No probabilistic CLV with uncertainty** — `calculate_simple_ltv` is a deterministic `(spend/days_active)*365*lifespan` formula, while lifetimes and PyMC-Marketing fit BG/NBD and Gamma-Gamma and return distributions. Buyers choosing on CLV accuracy will pick them; a point estimate cannot drive budget ranges. Effort M, because the Rust core would need a real likelihood fit and convergence diagnostics.
2. **No internal preprocessing, and the failure is documented but not guarded** — README lines 219-228 show 99.1% of real customers collapsing into one cluster on unscaled RFM, yet `AudienceSegmenter.fit()` still performs no standardization and emits no warning. scikit-learn's `Pipeline`/`StandardScaler` makes this the default-safe path. Effort S: scale or warn by default, keep an explicit opt-out.
3. **CLV and churn are not validated against either competitor** — `AUDIT_FINDINGS.md` lists "no statistical validation against lifetimes/PyMC-Marketing" and "no CLV accuracy benchmark"; the only evidence is a Spearman rank correlation of 0.7111 on 2,790 customers. scikit-learn and PyMC-Marketing ship model-selection and posterior-check tooling. Effort M.
4. **Mixed-type clustering is advertised but not delivered through the main API** — K-Prototypes support stops at numeric input in `AudienceSegmenter.fit()`, which silently degrades to KMeans. Users with channel/region categoricals must pre-encode, and the README says so, but the feature name promises more. Effort S-M.
5. **Distribution is not installable on most platforms from PyPI** — only a macOS ARM64 `cp39-abi3` wheel is published; Linux/Windows users fall back to compiling Rust. scikit-learn, PyMC-Marketing and lifetimes all publish broad wheels. This is an adoption blocker independent of algorithm quality. Effort S once Trusted Publishing is configured.

### Where we are differentiated
- One dependency covers RFM, clustering, churn, cohorts, lookalikes, lifecycle and segment SQL export; scikit-learn and the CLV libraries each cover one slice and require glue.
- Differential privacy and k-anonymity are implemented segmentation primitives, not a compliance add-on; none of the three competitors ships equivalent suppression/generalization.
- The Rust core is deterministic per seed and rayon-parallelized, and the repo publishes a real benchmark on 397,884 UCI Online Retail rows rather than synthetic timings.
- Segment output has a versioned Squid-CRM schema with an idempotency-key plan, which none of the comparison projects offers.

### Disposition
- **Build:** probabilistic CLV (BG/NBD/Gamma-Gamma) with intervals, and default-safe scaling/warnings in `AudienceSegmenter`; both close gaps users can measure.
- **Integrate/reuse:** keep scikit-learn as the optional preprocessing adapter and benchmark baseline; consider validating against PyMC-Marketing rather than reimplementing every BTYD variant.
- **Defer/reject:** do not turn this into a CDP, warehouse, dashboard or plugin framework (already out of scope); do not copy PyMC-Marketing's MMM/budget-optimizer scope.

### Sources
- https://github.com/scikit-learn/scikit-learn (1.9.1 release, accessed 2026-10-24)
- https://github.com/pymc-labs/pymc-marketing (1.3.0 release, accessed 2026-10-24)
- https://github.com/CamDavidsonPilon/lifetimes (archived; v0.11.3, accessed 2026-10-24)
- Repo files inspected: `README.md`, `COMPETITIVE_INTELLIGENCE.md`, `AUDIT_FINDINGS.md`, `pyproject.toml`, `docs/architecture/README.md`, `docs/ROADMAP_HONEST.md`
