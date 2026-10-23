# Squid segment contract v1

- **Status:** defined on the ClusterAudienceKit side; Squid acceptance and a live
  write test remain pending.
- **Contract ID:** `clusteraudiencekit.squid.segment.v1`
- **Owner:** ClusterAudienceKit for the output schema and record mapping;
  Squid-CRM for metadata acceptance and live verification.
- **Scope:** segment metadata and opaque membership only. External activation
  remains outside ClusterAudienceKit's engine; this file defines the contract a
  future adapter or caller can execute.

## Why two objects

Segments and memberships have different cardinality and update cadence. Writing
one object per segment and one object per membership avoids duplicating segment
metadata across every member and lets Squid update either side independently.

| Object | Purpose | Stable key |
|---|---|---|
| `audienceSegments` | Segment definition metadata | `segmentId` (`cak_seg_<24 hex>`) |
| `audienceSegmentMemberships` | Customer-to-segment membership | `membershipId` (`cak_mem_<24 hex>`) |

No consent decision is stored in either object. PyTagShield (or another consent
authority) remains responsible for deciding whether a write is allowed.

## Record schemas

- [`schemas/squid/v1/segment.schema.json`](../schemas/squid/v1/segment.schema.json)
- [`schemas/squid/v1/segment-membership.schema.json`](../schemas/squid/v1/segment-membership.schema.json)

### `audienceSegments`

| Field | Squid type | Required | Unique | Notes |
|---|---|---|---|---|
| `name` | auto | yes | no | Squid's automatic name field |
| `segmentId` | TEXT | yes | yes | Stable identity |
| `segmentType` | TEXT | yes | no | `rfm`, `cluster`, `behavioral`, `lookalike`, `custom` |
| `memberCount` | NUMBER | yes | no | Non-negative integer |
| `definitionHash` | TEXT | no | no | Optional reproducibility hash |
| `generatedAt` | DATE_TIME | yes | no | RFC3339 |
| `source` | TEXT | yes | no | Const `clusteraudiencekit` |
| `schemaVersion` | TEXT | yes | no | Const `1` |

### `audienceSegmentMemberships`

| Field | Squid type | Required | Unique | Notes |
|---|---|---|---|---|
| `name` | auto | yes | no | `<segmentId>:<customerId>` |
| `membershipId` | TEXT | yes | yes | Stable identity |
| `segmentId` | TEXT | yes | no | References `audienceSegments.segmentId` |
| `customerId` | TEXT | yes | no | Opaque CRM/customer reference; not email/phone/name |
| `score` | NUMBER | no | no | Optional model score |
| `rfmSegment` | TEXT | no | no | Optional business label |
| `assignedAt` | DATE_TIME | yes | no | RFC3339 |
| `source` | TEXT | yes | no | Const `clusteraudiencekit` |
| `schemaVersion` | TEXT | yes | no | Const `1` |

`customerId` values containing `@` are rejected by the contract helper so an
email address cannot accidentally become a membership key. Callers must still
enforce their own field allowlist before writing to Squid.

## Write plan

The helper emits a plan, not HTTP calls:

```python
from clusteraudiencekit.squid import segment_record, membership_record, write_plan

segment = segment_record(name="Champions", segment_type="rfm", member_count=42)
member = membership_record(segment=segment["segmentId"], customer_id="customer-7")
plan = write_plan([segment], [member])
```

Each batch uses the same two-phase sequence:

1. Look up the stable id:
   `GET /rest/audienceSegments?filter=segmentId[eq]:<segmentId>&limit=1`
   or `GET /rest/audienceSegmentMemberships?filter=membershipId[eq]:<membershipId>&limit=1`.
2. Add the existing Squid record `id` when the lookup matches.
3. `POST /rest/audienceSegments/upsert` or
   `POST /rest/audienceSegmentMemberships/upsert`.
4. Send
   `Idempotency-Key: clusteraudiencekit-segments-v1:<sha256(canonical batch body)>`.
   Reuse the same key when retrying an identical body. A `409` means the key
   was reused with different content; re-read the record before deciding.

Default batch size is 500 records per object, configurable from 1 to 5000.

## Squid acceptance checklist

1. Create both custom objects and the declared fields from `manifest()`.
2. Enforce uniqueness for `segmentId` and `membershipId` within a workspace.
3. Confirm `filter=<field>[eq]:<value>` lookup behavior and the batch-upsert
   response shape on a live/test workspace.
4. Confirm the API key needs create/update capability.
5. Run create, update, retry and body-hash-conflict cases against a live Squid
   instance.

## Explicit non-goals

- No HTTP client, credentials or live writes ship in ClusterAudienceKit.
- No generated typed GraphQL contract; REST is the interim path.
- No customer PII beyond the opaque `customerId`.
- No consent decision, legal basis or suppression verdict is encoded.
- No relation field is required; `segmentId` is a scalar lookup key so the
  contract does not depend on deferred custom-object relation DDL.