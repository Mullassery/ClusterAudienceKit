"""ClusterAudienceKit: High-performance audience segmentation library."""

import sys
import os
import importlib.util
from pathlib import Path

# Import SQL export functions from compiled Rust extension
try:
    import glob
    import importlib.machinery
    import importlib.util

    package_dir = Path(__file__).parent
    so_files = glob.glob(str(package_dir / "*.so"))

    if so_files:
        so_path = so_files[0]
        # Load the extension module using ExtensionFileLoader with the correct name
        # The .so exports PyInit_clusteraudiencekit, so we must use that exact name
        loader = importlib.machinery.ExtensionFileLoader("clusteraudiencekit", so_path)
        spec = importlib.util.spec_from_file_location("clusteraudiencekit", so_path, loader=loader)
        _rust_ext = importlib.util.module_from_spec(spec)
        sys.modules["_clusteraudiencekit_ext"] = _rust_ext  # Register with different name
        loader.exec_module(_rust_ext)

        export_segment_sql = _rust_ext.export_segment_sql
        export_all_segments_sql = _rust_ext.export_all_segments_sql
        get_supported_sql_dialects = _rust_ext.get_supported_sql_dialects
        get_segment_rfm_patterns = _rust_ext.get_segment_rfm_patterns
    else:
        raise ImportError("No .so files found")

except Exception as e:
    # Fallback if import fails
    _import_error = str(e)

    def export_segment_sql(*args, **kwargs):
        raise RuntimeError(f"SQL export not available: {_import_error}")

    def export_all_segments_sql(*args, **kwargs):
        raise RuntimeError(f"SQL export not available: {_import_error}")

    def get_supported_sql_dialects():
        raise RuntimeError(f"SQL export not available: {_import_error}")

    def get_segment_rfm_patterns(*args, **kwargs):
        raise RuntimeError(f"SQL export not available: {_import_error}")

# Optional: Try to import AudienceSegmenter if available
try:
    from ._core import PyAudienceSegmenter as AudienceSegmenter
except ImportError:
    AudienceSegmenter = None

__version__ = "5.9.0"
__author__ = "Georgi Mammen Mullassery"
__email__ = "mullassery@gmail.com"
__license__ = "MIT"

__all__ = [
    "AudienceSegmenter",
    "export_segment_sql",
    "export_all_segments_sql",
    "get_supported_sql_dialects",
    "get_segment_rfm_patterns",
    "__version__",
    "__author__",
    "__email__",
]
