"""Library-independent image-space tracking contracts."""
from .models import BoundingBox, FrameInfo, ImageSize, TrackingObservation
from .protocols import Tracker

__all__ = ["BoundingBox", "FrameInfo", "ImageSize", "TrackingObservation", "Tracker"]
