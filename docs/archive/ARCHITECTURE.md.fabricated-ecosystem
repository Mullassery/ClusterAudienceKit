# ClusterAudienceKit Architecture & Ecosystem Boundaries

## Mission

**Identify Who Matters**

Core Question: Which customers should we target?

## Core Responsibility

ClusterAudienceKit is **exclusively responsible** for:

- **Customer Segmentation** — RFM analysis, behavioral clustering
- **Audience Definition** — Rules, criteria, membership logic
- **Audience Calculation** — Computing audience membership
- **Audience Refresh** — Recalculating audience membership
- **Segment Hierarchy** — Parent/child segment relationships
- **Scoring Models** — Customer value, churn risk, engagement scores
- **Audience Analytics** — Segment performance, overlap analysis

## What We Do NOT Own

These belong to other products:

### ❌ Data Quality & Validation (StatGuardian)
- Schema validation
- Data freshness checks
- Drift detection
- Constraint enforcement

**Our role:** Assume input data is valid. Optionally require StatGuardian gates.

### ❌ Journey Orchestration (PyCustomerJourney)
- Journey definition
- Step execution
- Communications
- Attribution

**Our role:** Provide audiences. They activate journeys.

### ❌ Data Activation (PyReverseETL)
- Syncing to operational systems
- Destination management
- Sync orchestration

**Our role:** Define audiences. They sync membership.

### ❌ Query Optimization & Context Discovery (PyStreamMCP)
- Query planning for audience retrieval
- Intelligent data fetching
- Token/cost optimization
- Streaming retrieval

**Our role:** Use PyStreamMCP for efficient data ingestion and downstream member retrieval.

## Integration with PyStreamMCP

**IMPORTANT:** ClusterAudienceKit should use **PyStreamMCP** for:

### Ingestion (Data In)
```python
# ✅ CORRECT: Use PyStreamMCP for intelligent data retrieval
from pystreammcp import Discovery

# When building audiences from multiple sources:
discovery = Discovery.new(query_id="audience_ingestion")
sources = discovery.discover_sources()  # Find optimal data
optimized = discovery.optimize_for_cost()  # Efficient retrieval

# Use optimized context to build audience
audience = create_audience(
    name="churn_risk",
    criteria=optimized,  # PyStreamMCP's optimized context
    source_query=optimized
)
```

### Member Retrieval (Data Out)
```python
# ✅ CORRECT: Use PyStreamMCP for efficient member retrieval
from pystreammcp import Agent

# When retrieving audience members with context:
agent = Agent(agent_id="audience_member_context")
context = agent.query("customer context for audience enrichment")

# Get members with optimized context
members = audience.get_members()
enriched_members = enrich_with_context(members, context)  # Context from PyStreamMCP
```

### Why Use PyStreamMCP?
- **Intelligent Data Fetching:** Discover optimal data sources for audience building
- **Cost Optimization:** 60-75% reduction in token/data usage
- **Streaming Retrieval:** Progressive member retrieval for large audiences
- **Query Planning:** Handle complex multi-step audience logic
- **Token Efficiency:** Avoid redundant data fetches

**Do NOT rebuild these in ClusterAudienceKit:**
- Query planning
- Data source discovery
- Token optimization
- Streaming retrieval
- Cost estimation

## Architectural Principles

### 1. Segmentation First

ClusterAudienceKit owns segmentation logic. Everything else is delegated.

```
Customer Data
     ↓
PyStreamMCP (Discover & Optimize)
     ↓
Efficient Retrieval
     ↓
ClusterAudienceKit (Segmentation)
     ↓
Audiences
     ↓
PyReverseETL (Activate)
     ↓
Operational Systems
```

### 2. No Validation Logic

```rust
// ❌ NOT OUR JOB
fn validate_data(data: &Data) -> Result<()> {
    // Check schema
    // Check freshness
    // Check constraints
}

// ✅ OUR JOB
fn create_audience(data: &Data) -> Result<Audience> {
    // Segment data
    // Calculate membership
    // Return audience
}
```

### 3. Flexible Input

Audiences can be built from:
- Warehouse tables
- Query results
- APIs
- Stream data
- Real-time events

But don't rebuild query logic—use PyStreamMCP.

## Module Structure

```
core/src/
├── lib.rs                    # Public exports
├── error.rs                  # Error types
├── audience.rs              # Audience definition & membership
├── segment.rs               # Segmentation logic
├── clustering/              # Clustering algorithms
│   ├── kmeans.rs
│   ├── rfm.rs
│   └── behavioral.rs
├── scoring/                 # Scoring models
│   ├── churn_risk.rs
│   ├── lifetime_value.rs
│   └── engagement.rs
└── storage/                 # Persistence
    ├── schema.rs
    └── repository.rs
```

## Integration Points

### With PyStreamMCP (Ingestion)

```rust
// Use PyStreamMCP to discover optimal data sources
use pystreammcp::Discovery;

let discovery = Discovery::new(query_id);
let sources = discovery.discover_sources()?;
let optimized_query = discovery.optimize_for_cost()?;

// Build audience from optimized query
let audience = Audience::from_query(optimized_query);
```

### With PyStreamMCP (Member Retrieval)

```rust
// Use PyStreamMCP to efficiently retrieve member context
use pystreammcp::Agent;

let agent = Agent::new(agent_id);
let context = agent.query("member context")?;

// Enrich members with context
members.enrich_with(context);
```

### With StatGuardian

```rust
// Optionally validate input data
use statguardian::ValidationGate;

Audience {
    validation_gate: Option<ValidationGate>,
    // Only build audience if validation passes
}
```

### With PyReverseETL

```rust
// PyReverseETL syncs audience membership
sync_to_destination(
    audience=&audience,
    destination="salesforce",
)?;
```

### With PyCustomerJourney

```rust
// PyCustomerJourney launches journeys for audience members
launch_journey(
    audience_members=&audience.get_members(),
    journey="churn_prevention",
)?;
```

## Testability

Each boundary is enforced through:

1. **Module privacy** — Query/validation/journey modules don't exist
2. **Type system** — Can't express query logic in our types
3. **Documentation** — Explicit "use PyStreamMCP" statements
4. **Integration tests** — Test against PyStreamMCP/StatGuardian APIs

## Philosophy

ClusterAudienceKit is to customer segmentation what dbt is to transformation.

- dbt owns **transformation**, not validation
- Airflow owns **orchestration**, not transformation
- ClusterAudienceKit owns **segmentation**, not query optimization

Each product excels at its domain precisely because it doesn't try to own everything else.

PyStreamMCP handles query optimization so ClusterAudienceKit can focus entirely on segmentation.

## Outcome

When properly integrated:

```
Raw Data (Multiple Sources)
        ↓
PyStreamMCP optimizes retrieval
        ↓
Efficient Data Access
        ↓
ClusterAudienceKit segments
        ↓
Audience Membership
        ↓
PyReverseETL activates
        ↓
Operational Systems (CRM, Marketing, etc.)
        ↓
Customer Engagement
```

Each product does one thing excellently.
