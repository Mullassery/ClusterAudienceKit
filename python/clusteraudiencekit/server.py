"""REST API server for ClusterAudienceKit - integrates with workflow tools."""

from typing import Dict, Any, Optional, List


class ClusterAudienceKitServer:
    """REST API server for workflow integration."""

    def __init__(self, host: str = "0.0.0.0", port: int = 8002):
        """Initialize server."""
        self.host = host
        self.port = port
        self.audiences: Dict[str, Dict[str, Any]] = {}
        self.segments: Dict[str, Dict[str, Any]] = {}

    def create_audience(self, audience_id: str, config: Dict[str, Any]) -> Dict[str, Any]:
        """Create an audience."""
        self.audiences[audience_id] = {
            "id": audience_id,
            "name": config.get("name", audience_id),
            "criteria": config.get("criteria", {}),
            "status": "active",
            "member_count": 0,
        }
        return {
            "status": "success",
            "audience_id": audience_id,
            "message": f"Audience '{config.get('name')}' created",
        }

    def refresh_audience(self, audience_id: str, limit: Optional[int] = None) -> Dict[str, Any]:
        """Refresh audience membership."""
        if audience_id not in self.audiences:
            return {"status": "error", "message": f"Audience '{audience_id}' not found"}

        audience = self.audiences[audience_id]
        audience["member_count"] = limit or 1000

        return {
            "status": "success",
            "audience_id": audience_id,
            "members_calculated": audience["member_count"],
            "message": "Audience refreshed",
        }

    def get_audience_members(
        self, audience_id: str, limit: int = 100, offset: int = 0
    ) -> Dict[str, Any]:
        """Get audience members."""
        if audience_id not in self.audiences:
            return {"status": "error", "message": f"Audience '{audience_id}' not found"}

        audience = self.audiences[audience_id]
        member_count = audience.get("member_count", 0)
        members = [f"cust_{i}" for i in range(offset, min(offset + limit, member_count))]

        return {
            "status": "success",
            "audience_id": audience_id,
            "total_members": member_count,
            "returned": len(members),
            "members": members,
        }

    def create_segment(self, segment_id: str, config: Dict[str, Any]) -> Dict[str, Any]:
        """Create a segment."""
        self.segments[segment_id] = {
            "id": segment_id,
            "name": config.get("name", segment_id),
            "rules": config.get("rules", []),
            "status": "active",
        }
        return {
            "status": "success",
            "segment_id": segment_id,
            "message": f"Segment '{config.get('name')}' created",
        }

    def list_audiences(self) -> Dict[str, Any]:
        """List all audiences."""
        return {
            "status": "success",
            "audiences": list(self.audiences.values()),
            "count": len(self.audiences),
        }

    def list_segments(self) -> Dict[str, Any]:
        """List all segments."""
        return {
            "status": "success",
            "segments": list(self.segments.values()),
            "count": len(self.segments),
        }

    def get_metrics(self, audience_id: Optional[str] = None) -> Dict[str, Any]:
        """Get metrics."""
        if audience_id:
            if audience_id not in self.audiences:
                return {"status": "error", "message": "Audience not found"}
            audiences = [self.audiences[audience_id]]
        else:
            audiences = list(self.audiences.values())

        total_members = sum(a.get("member_count", 0) for a in audiences)

        return {
            "status": "success",
            "audience_id": audience_id,
            "total_audiences": len(audiences),
            "total_members": total_members,
        }

    def health_check(self) -> Dict[str, Any]:
        """Health check endpoint."""
        return {
            "status": "healthy",
            "service": "clusteraudiencekit",
            "version": "0.1.0",
            "audiences_count": len(self.audiences),
            "segments_count": len(self.segments),
        }


# Flask integration
def create_flask_app(server: Optional[ClusterAudienceKitServer] = None):
    """Create Flask app for REST API."""
    try:
        from flask import Flask, request, jsonify
    except ImportError:
        raise ImportError(
            "Flask is required for REST API. Install with: pip install flask"
        )

    app = Flask(__name__)
    srv = server or ClusterAudienceKitServer()

    @app.route("/health", methods=["GET"])
    def health():
        """Health check."""
        return jsonify(srv.health_check())

    @app.route("/audiences", methods=["GET"])
    def list_audiences():
        """List audiences."""
        return jsonify(srv.list_audiences())

    @app.route("/audiences", methods=["POST"])
    def create_audience():
        """Create audience."""
        data = request.get_json()
        audience_id = data.get("audience_id")
        config = data.get("config", {})

        if not audience_id:
            return (
                jsonify({"status": "error", "message": "audience_id required"}),
                400,
            )

        return jsonify(srv.create_audience(audience_id, config))

    @app.route("/audiences/<audience_id>/refresh", methods=["POST"])
    def refresh_audience(audience_id):
        """Refresh audience."""
        data = request.get_json() or {}
        limit = data.get("limit")
        return jsonify(srv.refresh_audience(audience_id, limit))

    @app.route("/audiences/<audience_id>/members", methods=["GET"])
    def get_members(audience_id):
        """Get audience members."""
        limit = request.args.get("limit", 100, type=int)
        offset = request.args.get("offset", 0, type=int)
        return jsonify(srv.get_audience_members(audience_id, limit, offset))

    @app.route("/segments", methods=["GET"])
    def list_segments():
        """List segments."""
        return jsonify(srv.list_segments())

    @app.route("/segments", methods=["POST"])
    def create_segment():
        """Create segment."""
        data = request.get_json()
        segment_id = data.get("segment_id")
        config = data.get("config", {})

        if not segment_id:
            return (
                jsonify({"status": "error", "message": "segment_id required"}),
                400,
            )

        return jsonify(srv.create_segment(segment_id, config))

    @app.route("/metrics", methods=["GET"])
    def metrics():
        """Get metrics."""
        audience_id = request.args.get("audience_id")
        return jsonify(srv.get_metrics(audience_id))

    return app


def run_server(host: str = "0.0.0.0", port: int = 8002):
    """Run the REST API server."""
    app = create_flask_app()
    app.run(host=host, port=port, debug=False)


if __name__ == "__main__":
    run_server()
