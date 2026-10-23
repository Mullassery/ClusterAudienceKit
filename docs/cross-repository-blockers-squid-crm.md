# Cross-Repository Blockers — ClusterAudienceKit ↔ Squid-CRM

Detailed specification of the ClusterAudienceKit → Squid-CRM integration
blocker, from the ClusterAudienceKit perspective. The shared context is
documented in `Mullassery/Squid-CRM/docs/cross-repository-blockers.md`; this
file records the ClusterAudienceKit side of the contract and what must change
here to unblock it.

Reference pin: `twentyhq/twenty@08a0e5f1ee2e7db7c1e6ef3aeec9bcef731f4e98`
(Twenty server `2.50.0`).

## 1. The blocker

**Status:** OPEN — No direct Squid-CRM adapter. Audience activation is expected
to pass through PyReverseETL, but no Squid payload mapping or custom-object
schema contract is defined. External platform activation is deliberately out of
scope for ClusterAudienceKit.

## 2. The two sides

| Aspect | ClusterAudienceKit (segmentation output) | Squid-CRM (destination) |
|--------|-----------------------------------------|-------------------------|
| Output | Customer segments, scores, RFM clusters, churn predictions, CLV | Custom-object REST `/rest/{object}`, generic JSON GraphQL CRUD |
| Schema | Segment definitions, audience membership, scores | Custom-object metadata (object + field definitions) |
| Activation path | SQL export to 8 warehouse dialects; expected to pass through PyReverseETL | Materialized physical tables + scalar/relation columns from metadata |
| Batch | SQL export; segment ROI, CAC, revenue attribution | Custom-object batch upsert (`/rest/{object}/upsert`) |
| Relationship | Segments reference customer identities | Relation fields (MANY_TO_ONE) create UUID columns + reverse ONE_TO_MANY |
| External activation | Deliberately out of scope for this library | CRM system of record |

## 3. What must be agreed (ClusterAudienceKit side)

1. **Custom-object schema contract.** What object name(s), field names/types,
   and relation fields will ClusterAudienceKit segments write to Squid? This
   must be defined before segments can be written to Squid without ad-hoc
   fields.
2. **Activation path.** ClusterAudienceKit → PyReverseETL → Squid, or direct
   ClusterAudienceKit → Squid custom-object REST? The expected path (via
   PyReverseETL) depends on the PyReverseETL idempotency blocker being resolved
   first.
3. **Field mapping.** Segment fields (segment_id, segment_name, customer_id,
   score, RFM class) must map to Squid custom-object field definitions.
4. **Batch/upsert.** Segmentation is batch-oriented. Squid's custom-object
   `/rest/{object}/upsert` supports this, but batch size, frequency, and
   idempotency key strategy must be agreed.
5. **Generated GraphQL.** If consumers need GraphQL access to custom objects,
   Squid's generated typed GraphQL schema for custom objects is deferred
   (known limitation). REST is the interim path.

## 4. Evidence

**ClusterAudienceKit:**
- Rust-powered segmentation engine with Python bindings.
- Covers RFM analysis, KMeans/K-Prototypes clustering, churn prediction, CLV,
  SQL export to 8 warehouse dialects, differential privacy / k-anonymity,
  real-time streaming segmentation, drift detection, lookalike audiences, and
  more — all real, tested Rust logic exposed through compiled Python extension.
- External platform activation (pushing segments to ad/CRM platforms) is
  deliberately out of scope.

**Squid-CRM:**
- Custom-object metadata writes create physical tables, scalar/relation columns,
  and generic REST CRUD — integration-tested.
- Custom-object batch upsert (`/rest/{object}/upsert`) is integration-tested.
- Generated GraphQL types for custom objects are not implemented.

## 5. Owner of next move

**ClusterAudienceKit** must define its output schema and agree on a
custom-object schema contract with Squid-CRM. Squid-CRM cannot pre-create
object/field metadata without knowing the shape. This blocker also depends on
the PyReverseETL idempotency blocker being resolved first (if activation flows
through PyReverseETL).

## 6. Related blockers

- **PyReverseETL + Squid:** activation payload + idempotency header — must be
  resolved before ClusterAudienceKit can activate through PyReverseETL.
- **PyTagShield + Squid:** consent decision contract — may affect whether
  segment data carries consent metadata.
- **PyCustomerHub + Squid:** shared event-schema contract — gates webhook
  delivery to the CDP.
