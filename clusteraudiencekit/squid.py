"""Versioned Squid-CRM custom-object contract for ClusterAudienceKit.

This module is deliberately pure data: it builds the metadata manifest, stable
record identities and a two-phase lookup/upsert plan. It performs no HTTP calls
and holds no credentials, so consumers can review the contract before wiring a
live Squid workspace.

Segments and membership are separate Squid custom objects:

- ``audienceSegments`` describes the segment (never customer PII).
- ``audienceSegmentMemberships`` links an opaque customer id to a segment.

Membership records never carry email, phone or names. Consent is not encoded
here; PyTagShield (or another consent authority) must decide whether activation
is allowed before any write.
"""

from __future__ import annotations

import hashlib
import json
from datetime import datetime, timezone
from typing import Any, Dict, Iterable, List, Mapping, Optional, Sequence

CONTRACT = "clusteraudiencekit.squid.segment.v1"
SCHEMA_VERSION = "1"
SEGMENT_TYPES = ("rfm", "cluster", "behavioral", "lookalike", "custom")
SEGMENT_OBJECT = "audienceSegments"
MEMBERSHIP_OBJECT = "audienceSegmentMemberships"
DEFAULT_BATCH_SIZE = 500

LIMITATIONS = [
    "This is a proposed contract, not a live Squid integration.",
    "ClusterAudienceKit does not perform Squid writes or hold Squid credentials.",
    "Records contain opaque customer ids only; no email, phone or name fields.",
    "Consent is not encoded in a record; PyTagShield remains the consent authority.",
    "Generated typed GraphQL for custom objects is not assumed; REST is the interim path.",
]


def _stable_id(prefix: str, *parts: str) -> str:
    payload = "\x1f".join(parts).encode("utf-8")
    return prefix + hashlib.sha256(payload).hexdigest()[:24]


def segment_id(name: str, segment_type: str, definition_hash: str = "") -> str:
    """Stable segment identity. Reuse the same inputs across runs."""
    if not name.strip():
        raise ValueError("segment name is required")
    if segment_type not in SEGMENT_TYPES:
        raise ValueError(f"unsupported segment_type '{segment_type}'")
    return _stable_id("cak_seg_", name.strip(), segment_type, definition_hash)


def membership_id(segment: str, customer_id: str) -> str:
    """Stable membership identity derived from segment + opaque customer id."""
    if not segment.startswith("cak_seg_"):
        raise ValueError("segment must be a cak_seg_ id")
    if not customer_id.strip():
        raise ValueError("customer_id is required")
    return _stable_id("cak_mem_", segment, customer_id.strip())


def _now_rfc3339() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def segment_record(
    *,
    name: str,
    segment_type: str,
    member_count: int,
    generated_at: Optional[str] = None,
    segment: Optional[str] = None,
    definition_hash: Optional[str] = None,
) -> Dict[str, Any]:
    """Build one ``audienceSegments`` record."""
    if member_count < 0:
        raise ValueError("member_count must be >= 0")
    segment = segment or segment_id(name, segment_type, definition_hash or "")
    record: Dict[str, Any] = {
        "name": name.strip(),
        "segmentId": segment,
        "segmentType": segment_type,
        "memberCount": int(member_count),
        "generatedAt": generated_at or _now_rfc3339(),
        "source": "clusteraudiencekit",
        "schemaVersion": SCHEMA_VERSION,
    }
    if definition_hash:
        record["definitionHash"] = definition_hash
    return record


def membership_record(
    *,
    segment: str,
    customer_id: str,
    assigned_at: Optional[str] = None,
    score: Optional[float] = None,
    rfm_segment: Optional[str] = None,
    membership: Optional[str] = None,
) -> Dict[str, Any]:
    """Build one ``audienceSegmentMemberships`` record.

    ``customer_id`` must be an opaque CRM/customer reference. Reject values that
    look like an email address so the record cannot silently become a PII store.
    """
    customer_id = customer_id.strip()
    if "@" in customer_id:
        raise ValueError("customer_id must be opaque, not an email address")
    membership = membership or membership_id(segment, customer_id)
    record: Dict[str, Any] = {
        "name": f"{segment}:{customer_id}",
        "membershipId": membership,
        "segmentId": segment,
        "customerId": customer_id,
        "assignedAt": assigned_at or _now_rfc3339(),
        "source": "clusteraudiencekit",
        "schemaVersion": SCHEMA_VERSION,
    }
    if score is not None:
        record["score"] = float(score)
    if rfm_segment:
        record["rfmSegment"] = rfm_segment.strip()
    return record


def manifest() -> Dict[str, Any]:
    """Squid custom-object metadata for segments and memberships."""
    segment_fields = [
        _field("segmentId", "Segment ID", "TEXT", unique=True, searchable=True),
        _field("segmentType", "Segment Type", "TEXT"),
        _field("memberCount", "Member Count", "NUMBER"),
        _field(
            "definitionHash",
            "Definition Hash",
            "TEXT",
            nullable=True,
            searchable=True,
        ),
        _field("generatedAt", "Generated At", "DATE_TIME"),
        _field("source", "Source", "TEXT"),
        _field("schemaVersion", "Schema Version", "TEXT"),
    ]
    membership_fields = [
        _field("membershipId", "Membership ID", "TEXT", unique=True, searchable=True),
        _field("segmentId", "Segment ID", "TEXT", searchable=True),
        _field("customerId", "Customer ID", "TEXT", searchable=True),
        _field("score", "Score", "NUMBER", nullable=True),
        _field("rfmSegment", "RFM Segment", "TEXT", nullable=True),
        _field("assignedAt", "Assigned At", "DATE_TIME"),
        _field("source", "Source", "TEXT"),
        _field("schemaVersion", "Schema Version", "TEXT"),
    ]
    return {
        "contract": CONTRACT,
        "schema_version": 1,
        "objects": [
            {
                "nameSingular": "audienceSegment",
                "namePlural": SEGMENT_OBJECT,
                "labelSingular": "Audience Segment",
                "labelPlural": "Audience Segments",
                "description": "ClusterAudienceKit generated segment metadata.",
                "auto_fields": ["name"],
                "unique_fields": ["segmentId"],
                "fields": segment_fields,
            },
            {
                "nameSingular": "audienceSegmentMembership",
                "namePlural": MEMBERSHIP_OBJECT,
                "labelSingular": "Audience Segment Membership",
                "labelPlural": "Audience Segment Memberships",
                "description": "Opaque customer-to-segment membership. No PII fields.",
                "auto_fields": ["name"],
                "unique_fields": ["membershipId"],
                "fields": membership_fields,
            },
        ],
        "limitations": LIMITATIONS,
    }


def _field(
    name: str,
    label: str,
    field_type: str,
    *,
    nullable: bool = False,
    unique: bool = False,
    searchable: bool = False,
) -> Dict[str, Any]:
    return {
        "name": name,
        "label": label,
        "type": field_type,
        "isNullable": nullable,
        "isUnique": unique,
        "isSearchable": searchable,
    }


def _chunks(items: Sequence[Mapping[str, Any]], size: int) -> Iterable[List[Mapping[str, Any]]]:
    for index in range(0, len(items), size):
        yield list(items[index : index + size])


def _canonical(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def write_plan(
    segments: Sequence[Mapping[str, Any]],
    memberships: Sequence[Mapping[str, Any]],
    *,
    batch_size: int = DEFAULT_BATCH_SIZE,
) -> Dict[str, Any]:
    """Build the two-phase lookup/upsert plan. No network calls are made."""
    if batch_size < 1 or batch_size > 5000:
        raise ValueError("batch_size must be in 1..=5000")
    segment_batches = list(_chunks(segments, batch_size))
    membership_batches = list(_chunks(memberships, batch_size))
    operations: List[Dict[str, Any]] = []
    for batch in segment_batches:
        operations.append(
            {
                "object": SEGMENT_OBJECT,
                "match_field": "segmentId",
                "lookup": {
                    "method": "GET",
                    "path": f"/rest/{SEGMENT_OBJECT}",
                    "query": "filter=segmentId[eq]:<segmentId>&limit=1",
                    "selection": "exact segmentId match only",
                },
                "upsert": {
                    "method": "POST",
                    "path": f"/rest/{SEGMENT_OBJECT}/upsert",
                },
                "records": batch,
            }
        )
    for batch in membership_batches:
        operations.append(
            {
                "object": MEMBERSHIP_OBJECT,
                "match_field": "membershipId",
                "lookup": {
                    "method": "GET",
                    "path": f"/rest/{MEMBERSHIP_OBJECT}",
                    "query": "filter=membershipId[eq]:<membershipId>&limit=1",
                    "selection": "exact membershipId match only",
                },
                "upsert": {
                    "method": "POST",
                    "path": f"/rest/{MEMBERSHIP_OBJECT}/upsert",
                },
                "records": batch,
            }
        )

    body = {"segments": list(segments), "memberships": list(memberships)}
    idempotency_key = (
        "clusteraudiencekit-segments-v1:"
        + hashlib.sha256(_canonical(body).encode("utf-8")).hexdigest()
    )
    return {
        "contract": CONTRACT,
        "schema_version": 1,
        "manifest": manifest(),
        "batch_size": batch_size,
        "operations": operations,
        "idempotency": {
            "header": "Idempotency-Key",
            "value": idempotency_key,
            "scope": "Squid workspace",
            "semantics": "stable for an identical batch body; reuse on retry",
        },
        "activation_sequence": [
            "Create the audienceSegments and audienceSegmentMemberships metadata from the manifest.",
            "Look up each stable id and add the existing Squid record id before upsert.",
            "POST each batch to the matching /rest/<object>/upsert path with the Idempotency-Key.",
            "Treat 409 as a body-hash conflict; re-read the record before deciding whether to retry.",
        ],
        "limitations": LIMITATIONS,
    }