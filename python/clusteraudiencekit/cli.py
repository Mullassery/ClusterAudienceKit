"""CLI for ClusterAudienceKit - integration with workflow tools."""

import json
import sys
from typing import Optional


class CLIInterface:
    """Command-line interface for ClusterAudienceKit workflow integration."""

    def __init__(self):
        self.audiences = {}
        self.segments = {}

    def create_audience(
        self,
        audience_id: str,
        name: str,
        criteria: Optional[dict] = None,
    ) -> dict:
        """Create a new audience.

        Args:
            audience_id: Unique audience identifier
            name: Human-readable audience name
            criteria: Audience selection criteria

        Returns:
            JSON response with audience details
        """
        self.audiences[audience_id] = {
            "id": audience_id,
            "name": name,
            "criteria": criteria or {},
            "status": "active",
            "member_count": 0,
        }
        return {
            "status": "success",
            "audience_id": audience_id,
            "name": name,
            "message": f"Audience '{name}' created successfully",
        }

    def refresh_audience(self, audience_id: str, limit: Optional[int] = None) -> dict:
        """Refresh/recalculate audience membership.

        Args:
            audience_id: Audience to refresh
            limit: Optional limit on members to process

        Returns:
            JSON response with refresh results
        """
        if audience_id not in self.audiences:
            return {
                "status": "error",
                "message": f"Audience '{audience_id}' not found",
            }

        audience = self.audiences[audience_id]
        # Simulate refresh
        audience["member_count"] = limit or 1000
        audience["last_refreshed"] = True

        return {
            "status": "success",
            "audience_id": audience_id,
            "members_calculated": audience["member_count"],
            "message": f"Audience refreshed with {audience['member_count']} members",
        }

    def get_audience_members(
        self, audience_id: str, limit: int = 100, offset: int = 0
    ) -> dict:
        """Get members of an audience.

        Args:
            audience_id: Audience identifier
            limit: Max members to return
            offset: Pagination offset

        Returns:
            JSON response with member list
        """
        if audience_id not in self.audiences:
            return {
                "status": "error",
                "message": f"Audience '{audience_id}' not found",
            }

        audience = self.audiences[audience_id]
        # Simulate member retrieval
        member_count = audience.get("member_count", 0)
        members = [f"cust_{i}" for i in range(offset, min(offset + limit, member_count))]

        return {
            "status": "success",
            "audience_id": audience_id,
            "total_members": member_count,
            "returned": len(members),
            "members": members,
            "message": f"Retrieved {len(members)} members from audience",
        }

    def create_segment(
        self,
        segment_id: str,
        name: str,
        rules: Optional[list] = None,
    ) -> dict:
        """Create a new segment.

        Args:
            segment_id: Unique segment identifier
            name: Human-readable segment name
            rules: Segmentation rules

        Returns:
            JSON response with segment details
        """
        self.segments[segment_id] = {
            "id": segment_id,
            "name": name,
            "rules": rules or [],
            "status": "active",
        }
        return {
            "status": "success",
            "segment_id": segment_id,
            "name": name,
            "message": f"Segment '{name}' created successfully",
        }

    def list_audiences(self) -> dict:
        """List all audiences.

        Returns:
            JSON response with audience list
        """
        audiences = list(self.audiences.values())
        return {
            "status": "success",
            "audiences": audiences,
            "count": len(audiences),
        }

    def list_segments(self) -> dict:
        """List all segments.

        Returns:
            JSON response with segment list
        """
        segments = list(self.segments.values())
        return {
            "status": "success",
            "segments": segments,
            "count": len(segments),
        }

    def get_metrics(self, audience_id: Optional[str] = None) -> dict:
        """Get audience metrics.

        Args:
            audience_id: Optional specific audience ID

        Returns:
            JSON response with metrics
        """
        if audience_id:
            if audience_id not in self.audiences:
                return {"status": "error", "message": f"Audience not found"}
            audiences = [self.audiences[audience_id]]
        else:
            audiences = list(self.audiences.values())

        total_members = sum(a.get("member_count", 0) for a in audiences)
        active_audiences = sum(1 for a in audiences if a.get("status") == "active")

        return {
            "status": "success",
            "audience_id": audience_id,
            "total_audiences": len(audiences),
            "active_audiences": active_audiences,
            "total_members": total_members,
        }


def main():
    """Main CLI entry point."""
    cli = CLIInterface()

    if len(sys.argv) < 2:
        print_help()
        sys.exit(1)

    command = sys.argv[1]

    try:
        if command == "create-audience":
            if len(sys.argv) < 4:
                print(json.dumps({"error": "Missing audience_id or name"}))
                sys.exit(1)

            audience_id = sys.argv[2]
            name = sys.argv[3]

            result = cli.create_audience(audience_id, name)
            print(json.dumps(result))

        elif command == "refresh-audience":
            if len(sys.argv) < 3:
                print(json.dumps({"error": "Missing audience_id"}))
                sys.exit(1)

            audience_id = sys.argv[2]
            limit = int(sys.argv[3]) if len(sys.argv) > 3 else None

            result = cli.refresh_audience(audience_id, limit)
            print(json.dumps(result))

        elif command == "get-members":
            if len(sys.argv) < 3:
                print(json.dumps({"error": "Missing audience_id"}))
                sys.exit(1)

            audience_id = sys.argv[2]
            limit = int(sys.argv[3]) if len(sys.argv) > 3 else 100
            offset = int(sys.argv[4]) if len(sys.argv) > 4 else 0

            result = cli.get_audience_members(audience_id, limit, offset)
            print(json.dumps(result))

        elif command == "create-segment":
            if len(sys.argv) < 4:
                print(json.dumps({"error": "Missing segment_id or name"}))
                sys.exit(1)

            segment_id = sys.argv[2]
            name = sys.argv[3]

            result = cli.create_segment(segment_id, name)
            print(json.dumps(result))

        elif command == "list-audiences":
            result = cli.list_audiences()
            print(json.dumps(result))

        elif command == "list-segments":
            result = cli.list_segments()
            print(json.dumps(result))

        elif command == "metrics":
            audience_id = sys.argv[2] if len(sys.argv) > 2 else None
            result = cli.get_metrics(audience_id)
            print(json.dumps(result))

        elif command == "help":
            print_help()

        else:
            print(json.dumps({"error": f"Unknown command: {command}"}))
            sys.exit(1)

    except Exception as e:
        print(json.dumps({"error": str(e), "status": "error"}))
        sys.exit(1)


def print_help():
    """Print help message."""
    help_text = """
ClusterAudienceKit CLI - Audience & Segmentation Workflow Integration

USAGE:
    clusteraudiencekit <command> [options]

COMMANDS:
    create-audience <audience_id> <name>
        Create a new audience
        - audience_id: Unique identifier (required)
        - name: Human-readable name (required)

        Example:
            clusteraudiencekit create-audience churn_risk "High Churn Risk Customers"

    refresh-audience <audience_id> [limit]
        Refresh/recalculate audience membership
        - audience_id: Audience identifier (required)
        - limit: Max members to process (optional)

        Example:
            clusteraudiencekit refresh-audience churn_risk 5000

    get-members <audience_id> [limit] [offset]
        Get members of an audience (paginated)
        - audience_id: Audience identifier (required)
        - limit: Max members to return (default: 100)
        - offset: Pagination offset (default: 0)

        Example:
            clusteraudiencekit get-members churn_risk 100 0

    create-segment <segment_id> <name>
        Create a new segment
        - segment_id: Unique identifier (required)
        - name: Human-readable name (required)

        Example:
            clusteraudiencekit create-segment high_value "High Value Customers"

    list-audiences
        List all audiences

        Example:
            clusteraudiencekit list-audiences

    list-segments
        List all segments

        Example:
            clusteraudiencekit list-segments

    metrics [audience_id]
        Get audience metrics
        - audience_id: Optional specific audience ID

        Example:
            clusteraudiencekit metrics churn_risk

    help
        Show this help message

OUTPUT FORMAT:
    All commands return JSON output for easy parsing in workflows

EXAMPLES FOR WORKFLOW TOOLS:

n8n HTTP Request:
  URL: POST http://localhost/api/audiences
  Body: { "audience_id": "churn_risk", "name": "Churn Risk" }

Power Automate HTTP:
  POST to: http://localhost/api/audiences
  Headers: Content-Type: application/json
  Body: { "audience_id": "@{triggerBody()?['audience_id']}", ... }

Bash/Shell Script:
  clusteraudiencekit create-audience churn_risk "Churn Risk" | jq '.audience_id'
  clusteraudiencekit get-members churn_risk 100 | jq '.members[]'

Temporal Workflow:
  Use HTTP connector to POST to REST API endpoints
"""
    print(help_text)


if __name__ == "__main__":
    main()
