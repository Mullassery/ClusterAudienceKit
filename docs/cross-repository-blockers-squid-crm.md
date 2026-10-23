# Cross-Repository Blockers — ClusterAudienceKit ↔ Squid-CRM

Detailed specification of the ClusterAudienceKit → Squid-CRM integration
blocker, from the ClusterAudienceKit perspective. The shared context is
documented in `Mullassery/Squid-CRM/docs/cross-repository-blockers.md`; this
file records the ClusterAudienceKit side of the contract and what must change
here to unblock it.

Reference pin: `twentyhq/twenty@08a0e5f1ee2e7db7c1e6ef3aeec9bcef731f4e98`
(Twenty server `2.50.0`).

## 1. The blocker

**Status:** PARTIAL — ClusterAudienceKit has defined the versioned Squid
custom-object contract, schemas and pure-data write plan. There is still no HTTP
client or live Squid write; Squid acceptance and the live test remain. External
platform activation stays deliberately out of scope for the engine.

## 2. The two sides

| Aspect | ClusterAudienceKit (segmentation output) | Squid-CRM (destination) |
|--------|-----------------------------------------|-------------------------|
| Output | Customer segments, scores, RFM clusters, churn predictions, CLV | Custom-object REST `/rest/{object}`, generic JSON GraphQL CRUD |
| Schema | Segment definitions, audience membership, scores | Custom-object metadata (object + field definitions) |
| Activation path | SQL export to 8 warehouse dialects; expected to pass through PyReverseETL | Materialized physical tables + scalar/relation columns from metadata |
| Batch | SQL export; segment ROI, CAC, revenue attribution | Custom-object batch upsert (`/rest/{object}/upsert`) |
| Relationship | Segments reference customer identities | Relation fields (MANY_TO_ONE) create UUID columns + reverse ONE_TO_MANY |
| External activation | Deliberately out of scope for this library | CRM system of record |

## 3. ClusterAudienceKit-side contract (defined)

1. **Custom-object schema.** Two objects:
   `audienceSegments` (`segmentId` unique) and
   `audienceSegmentMemberships` (`membershipId` unique). Schemas live in
   `schemas/squid/v1/segment.schema.json` and
   `schemas/squid/v1/segment-membership.schema.json`.
2. **Activation path.** Direct custom-object REST is the reference path via a
   pure-data plan; a future adapter may also use PyReverseETL. The engine itself
   still does not perform external activation.
3. **Field mapping.** Segment metadata is separate from membership records;
   `customerId` is an opaque CRM reference and email-like values are rejected.
4. **Batch/upsert.** Two-phase stable-id lookup + `/rest/<object>/upsert`, with a
   deterministic `Idempotency-Key` based on the canonical batch body.
5. **Generated GraphQL.** Not assumed; REST is the interim path.

## 4. Evidence

**ClusterAudienceKit:**
- Rust-powered segmentation engine with Python bindings.
- Covers RFM analysis, KMeans/K-Prototypes clustering, churn prediction, CLV,
  SQL export to 8 warehouse dialects, differential privacy / k-anonymity,
  real-time streaming segmentation, drift detection, lookalike audiences, and
  more — all real, tested Rust logic exposed through compiled Python extension.
- External platform activation remains out of scope for the engine; the
  contract and write-plan generator are pure data with no HTTP client.

**Squid-CRM:**
- Custom-object metadata writes create physical tables, scalar/relation columns,
  and generic REST CRUD — integration-tested.
- Custom-object batch upsert (`/rest/{object}/upsert`) is integration-tested.
- Generated GraphQL types for custom objects are not implemented.

## 5. Owner of next move

**ClusterAudienceKit:** defined the contract, schemas, manifest and write plan
in `clusteraudiencekit/squid.py` and `docs/SQUID_SEGMENTS.md`.
**Squid-CRM:** must accept/amend the metadata contract and run the live
create/update/retry/conflict test.

## 6. Related blockers

- **PyReverseETL + Squid:** activation payload + idempotency header — the
  PyReverseETL side is implemented (person/company create path); custom-object
  flow is not yet wired through it.
- **PyTagShield + Squid:** consent decision contract — may affect whether
  segment data carries consent metadata.
- **PyCustomerHub + Squid:** shared event-schema contract — gates webhook
  delivery to the CDP.
