# ClusterAudienceKit Roadmap

**Current Version:** v1.0.0  
**Last Updated:** July 2026  
**Status:** Stable for RFM + KMeans segmentation; advanced methods in development

---

## Known Limitations (v1.0.0)

### 🔴 Blocking Issues
None identified.

### 🟡 Experimental Features
- **K-Prototypes (mixed data types):** 5 TODOs found in code
  - [ ] Implementation incomplete; not tested on production data
  - [ ] Performance characteristics unknown
  - **Impact:** Only use RFM + KMeans; K-Prototypes will fail
  - **Fix timeline:** v1.2.0 (Q3 2026)

- **Streaming/incremental updates:** Listed in README but not shipped
  - [ ] Structure exists; algorithm not implemented
  - [ ] Can't add new customers without full refit
  - **Impact:** Batch processing only; refit entire dataset for updates
  - **Fix timeline:** v1.3.0 (Q4 2026)

- **Drift detection:** Marked incomplete in code
  - [ ] Skeleton structure exists
  - [ ] Statistical tests not wired
  - **Impact:** No built-in drift monitoring; use external tools
  - **Fix timeline:** v1.2.0 (Q3 2026)

- **Customer Lifetime Value (CLV):** Listed on roadmap, not implemented
  - [ ] Not shipped in v1.0.0
  - **Impact:** Use external libraries (lifetimes, etc.)
  - **Fix timeline:** v2.0.0 (Q1 2027)

### 🟢 Shipping/Stable (v1.0.0)
- ✅ RFM + KMeans segmentation
- ✅ Segment profiling
- ✅ Silhouette scoring
- ✅ Model persistence (save/load)
- ✅ Performance optimized for 100k-1M customers

---

## 🔒 Security Issues (See SECURITY_AUDIT.md)

### HIGH — v1.0.1
- [ ] **Pin all dependency versions** (0 pinned, 12 floating)

### MEDIUM — v1.1.0
- [ ] **Input validation** (Pydantic models for data validation)
- [ ] **Resource limits** (prevent DoS via large clustering jobs)

---

## TODOs in Code
5 found in various files related to:
- K-Prototypes implementation
- Streaming updates
- Drift detection

---

## Roadmap

### v1.0.1 (Q3 2026) — Documentation
- [ ] Add performance benchmark methodology
- [ ] Document RFM calculation details
- [ ] Add example use cases

### v1.1.0 (Q3 2026) — Quality Improvements
- [ ] Additional silhouette score explanations
- [ ] Segment stability metrics
- [ ] Better default cluster selection

### v1.2.0 (Q3 2026) — Drift Detection + K-Prototypes
- [ ] Implement drift detection (segment distribution changes)
- [ ] K-Prototypes for categorical + numerical data
- [ ] Mixed data type handling

### v1.3.0 (Q4 2026) — Streaming Support
- [ ] Incremental model updates
- [ ] Add new customers without refit
- [ ] Streaming segment assignment

### v2.0.0 (Q1 2027) — Advanced Analytics
- [ ] Customer Lifetime Value (CLV)
- [ ] Churn prediction per segment
- [ ] Segment optimization recommendations
- [ ] Integration with Martech platforms (HubSpot, Segment)

---

## Performance Notes

Benchmark claims (46x-1000x faster than scikit-learn) are Apple M1-specific. Your results depend on hardware. Recommend profiling on your actual data before production deployment.

---

## Not Planned
- Real-time streaming ingestion (batch updates only)
- Hierarchical clustering methods
- GPU acceleration
