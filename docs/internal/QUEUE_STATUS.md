# Queue status — 2026-10-19

This file records what was shipped, what is blocked/failed, and what is rejected for this repository. It complements `AUDIT_FINDINGS.md`.

## Shipped
- Optional sklearn `PreprocessingPipeline` and `SegmenterWithPreprocessing` adapter (`[preprocessing]` extra); audit/bundle docs added.

## Blocked / failed
- Linux/Windows wheels.
- CLV/churn validation against lifetimes/PyMC-Marketing.

## Rejected / deferred
- A full CDP or warehouse; integrate with MessageBirds/PyReverseETL.

## Policy

Any future feature or upgrade must check `AUDIT_FINDINGS.md` and this file before implementation. A blocked item is retried only when its external dependency is available; a rejected item needs new user evidence or a design change to reopen.
