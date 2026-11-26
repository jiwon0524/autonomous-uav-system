"""Camera input contracts and optional OpenCV adapter; no eager CV imports."""
from .models import CameraFrame
from .protocols import CameraSource, CameraSourceError, SourceOpenError, SourceClosedError, FrameReadError
from .opencv import OpenCVSource

__all__ = ["CameraFrame", "CameraSource", "CameraSourceError", "SourceOpenError",
           "SourceClosedError", "FrameReadError", "OpenCVSource"]
