"""OKF Segment Profiles for ClusterAudienceKit.

Segment characteristics, stability tracking, and cross-org benchmarks
for audience segmentation and customer intelligence.
"""

from pathlib import Path
from typing import Dict, List, Optional
import json
from dataclasses import dataclass
from datetime import datetime


@dataclass
class SegmentProfile:
    """Audience segment profile."""

    segment_id: str
    name: str
    size: int
    characteristics: Dict
    stability_score: float  # 0-100 (higher = more stable)
    churn_rate: float
    last_measured: str


class OKFSegmentProfiles:
    """Manage segment profiles and benchmarks."""

    def __init__(self, profiles_dir: Path = None):
        self.profiles_dir = profiles_dir or Path.cwd() / "segment_profiles"
        self.profiles_dir.mkdir(exist_ok=True)

    def save_segment(self, profile: SegmentProfile) -> None:
        """Save segment profile."""
        filename = f"segment_{profile.segment_id}.json"
        with open(self.profiles_dir / filename, 'w') as f:
            json.dump({
                'segment_id': profile.segment_id,
                'name': profile.name,
                'size': profile.size,
                'characteristics': profile.characteristics,
                'stability_score': profile.stability_score,
                'churn_rate': profile.churn_rate,
                'last_measured': profile.last_measured
            }, f, indent=2)

    def get_segment(self, segment_id: str) -> Optional[SegmentProfile]:
        """Get segment profile."""
        filename = f"segment_{segment_id}.json"
        filepath = self.profiles_dir / filename

        if not filepath.exists():
            return None

        with open(filepath) as f:
            data = json.load(f)
            return SegmentProfile(**data)

    def get_stable_segments(self, min_stability: float = 70.0) -> List[Dict]:
        """Get segments with high stability."""
        stable_segments = []

        for f in self.profiles_dir.glob("segment_*.json"):
            with open(f) as fp:
                data = json.load(fp)
                if data['stability_score'] >= min_stability:
                    stable_segments.append({
                        'segment_id': data['segment_id'],
                        'name': data['name'],
                        'stability': data['stability_score'],
                        'size': data['size']
                    })

        return sorted(stable_segments, key=lambda x: x['stability'], reverse=True)

    def compare_with_industry(self, segment_id: str) -> Optional[Dict]:
        """Compare segment metrics to industry benchmarks."""
        segment = self.get_segment(segment_id)
        if not segment:
            return None

        # Industry benchmarks (example)
        benchmarks = {
            'avg_segment_size': 50000,
            'avg_stability': 65.0,
            'avg_churn_rate': 0.05
        }

        return {
            'segment_name': segment.name,
            'vs_industry_size': (segment.size / benchmarks['avg_segment_size']) * 100,
            'vs_industry_stability': segment.stability_score - benchmarks['avg_stability'],
            'vs_industry_churn': segment.churn_rate - benchmarks['avg_churn_rate']
        }
