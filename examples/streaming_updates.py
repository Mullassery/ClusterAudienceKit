"""Real-time streaming segmentation example.

This replaces a previous version of this file that was entirely a
commented-out sketch of a fictional API (`AudienceSegmenter(method=...)`,
`.update()`, `.segment_stability()`) that never existed. The streaming
engine shown below (`StreamingSegmentationEngine`) is real, wired to Python,
and covered by `tests/test_wired_modules.py::TestStreaming` — this example
mirrors that test, not aspirational code.
"""

from clusteraudiencekit import (
    StreamingConfig,
    StreamingEvent,
    StreamingSegmentationEngine,
)


def main():
    print("ClusterAudienceKit - Streaming Updates Example")
    print("=" * 50)

    # batch_size/window control how the engine buffers and aggregates
    # incoming events before recomputing RFM state per customer.
    config = StreamingConfig(batch_size=10, window="hour")
    engine = StreamingSegmentationEngine(config)

    print("\n1. Processing individual events...")
    event = StreamingEvent("cust_1", "purchase", 500.0, 1704067200)
    update = engine.process_event(event)
    print(f"   Processed event for {update.customer_id}")
    print(f"   Current segment: {engine.get_segment('cust_1')}")

    print("\n2. Processing a batch of events...")
    events = [
        StreamingEvent("cust_1", "purchase", 100.0, 1704067200),
        StreamingEvent("cust_2", "engagement", 0.0, 1704067200),
        StreamingEvent("cust_3", "purchase", 25.0, 1704067200),
    ]
    updates = engine.process_batch(events)
    print(f"   Processed {len(updates)} events in the batch")

    print("\n3. Current state...")
    print(f"   Tracked customers: {engine.customer_count()}")
    print(f"   Segment distribution: {engine.segment_distribution()}")

    print("\n" + "=" * 50)
    print(
        "Note: drift-triggered re-clustering (ReclusterConfig) is also real\n"
        "and covered by src/engine/streaming.rs tests, but is not shown here\n"
        "to keep this example short. See docs/ROADMAP_HONEST.md for details."
    )


if __name__ == "__main__":
    main()
