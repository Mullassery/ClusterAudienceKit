# ClusterAudienceKit MCP 2.0 Quick Start

> AI-native audience segmentation. Ask Claude to cluster customers, predict churn, calculate CLV, generate lookalikes.

## Installation

```bash
pip install clusteraudiencekit>=5.9
```

## Basic Usage

```python
from clusteraudiencekit import AudienceSegmenter

# Create segmenter
segmenter = AudienceSegmenter()

# Enable MCP (starts on port 8768)
endpoint = segmenter.start_mcp_connector()
print(f"MCP endpoint: {endpoint}")

# Claude can now:
# - "Segment my customers into 5 groups"
# - "Show me at-risk customers who might churn"
# - "Generate a lookalike audience from our VIP segment"
# - "Calculate customer lifetime value"
# - "Find the optimal number of segments"
```

## 10 MCP Tools

1. `segment_customers` — Cluster using RFM, K-Means, or auto-K
2. `get_segment_profiles` — View segment metadata
3. `export_segments_sql` — Generate SQL for warehouse
4. `sync_to_platform` — Send to Braze, Klaviyo, Segment
5. `detect_segment_drift` — Find changing segments
6. `estimate_auto_k` — Recommend cluster count
7. `calculate_customer_lifetime_value` — Predict CLV
8. `predict_churn` — Identify at-risk customers
9. `generate_lookalike_audience` — Find similar customers
10. `cohort_analysis` — Retention by segment

## Example Queries for Claude

```
"Segment my 1M customers into high-value, medium, and at-risk groups"
→ Uses segment_customers tool

"Which segments have the highest churn risk?"
→ Uses predict_churn + get_segment_profiles

"Generate a lookalike audience for our VIP customers (10% of base)"
→ Uses generate_lookalike_audience

"Export segments to Snowflake"
→ Uses export_segments_sql

"What's the optimal number of customer clusters?"
→ Uses estimate_auto_k

"Show me CLV distribution across segments"
→ Uses calculate_customer_lifetime_value
```

## Configuration

Create `clusteraudiencekit.toml`:

```toml
[mcp]
enabled = true
port = 8768
discovery_method = "stdio"

[mcp.tools]
segment_customers = true
predict_churn = true
# ... enable/disable specific tools
```

---

For full documentation, see [README.md](README.md)
