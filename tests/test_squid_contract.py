"""Tests for the pure-data Squid custom-object contract."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = ROOT / "clusteraudiencekit" / "squid.py"
spec = importlib.util.spec_from_file_location("cak_squid_contract", MODULE_PATH)
squid = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(squid)


def test_ids_are_stable_and_distinct():
    one = squid.segment_id("Champions", "rfm", "def-1")
    two = squid.segment_id("Champions", "rfm", "def-1")
    other = squid.segment_id("Champions", "cluster", "def-1")
    assert one == two
    assert one.startswith("cak_seg_")
    assert len(one) == len("cak_seg_") + 24
    assert other != one
    membership = squid.membership_id(one, "customer-7")
    assert membership == squid.membership_id(one, "customer-7")
    assert membership.startswith("cak_mem_")


def test_segment_and_membership_records_are_pii_free():
    segment = squid.segment_record(
        name="Champions",
        segment_type="rfm",
        member_count=42,
        generated_at="2026-10-23T00:00:00Z",
        definition_hash="def-1",
    )
    assert segment["source"] == "clusteraudiencekit"
    assert segment["schemaVersion"] == "1"
    assert segment["memberCount"] == 42
    assert "email" not in segment

    membership = squid.membership_record(
        segment=segment["segmentId"],
        customer_id="customer-7",
        assigned_at="2026-10-23T00:00:00Z",
        score=0.91,
        rfm_segment="Champions",
    )
    assert membership["segmentId"] == segment["segmentId"]
    assert membership["score"] == 0.91
    assert "email" not in membership


def test_email_like_customer_id_is_rejected():
    segment = squid.segment_id("Champions", "rfm")
    try:
        squid.membership_record(segment=segment, customer_id="a@b.c")
    except ValueError as error:
        assert "opaque" in str(error)
    else:
        raise AssertionError("email-like customer id must be rejected")


def test_manifest_defines_both_custom_objects():
    manifest = squid.manifest()
    names = {obj["namePlural"] for obj in manifest["objects"]}
    assert names == {"audienceSegments", "audienceSegmentMemberships"}
    assert manifest["objects"][0]["unique_fields"] == ["segmentId"]
    assert manifest["objects"][1]["unique_fields"] == ["membershipId"]


def test_write_plan_batches_and_is_deterministic():
    segments = [
        squid.segment_record(
            name=f"Segment {i}",
            segment_type="cluster",
            member_count=i,
            generated_at="2026-10-23T00:00:00Z",
        )
        for i in range(3)
    ]
    memberships = [
        squid.membership_record(
            segment=segments[0]["segmentId"],
            customer_id=f"customer-{i}",
            assigned_at="2026-10-23T00:00:00Z",
        )
        for i in range(3)
    ]
    plan_a = squid.write_plan(segments, memberships, batch_size=2)
    plan_b = squid.write_plan(segments, memberships, batch_size=2)
    assert plan_a["idempotency"]["value"] == plan_b["idempotency"]["value"]
    assert plan_a["idempotency"]["header"] == "Idempotency-Key"
    objects = [op["object"] for op in plan_a["operations"]]
    assert objects.count("audienceSegments") == 2
    assert objects.count("audienceSegmentMemberships") == 2
    assert plan_a["operations"][0]["lookup"]["query"].startswith("filter=segmentId")


def test_schema_files_parse_and_require_stable_ids():
    for name, required in (
        ("segment.schema.json", "segmentId"),
        ("segment-membership.schema.json", "membershipId"),
    ):
        schema = json.loads((ROOT / "schemas" / "squid" / "v1" / name).read_text())
        assert required in schema["required"]
        assert schema["additionalProperties"] is False
