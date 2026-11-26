"""Contracts only: concrete CV adapters are intentionally not implemented."""
from typing import Protocol, TypeVar, runtime_checkable
from uuid import UUID

from .models import BoundingBox, FrameInfo, TrackingObservation

FrameT = TypeVar("FrameT", contravariant=True)


@runtime_checkable
class Tracker(Protocol[FrameT]):
    """The caller provides an already selected Target Manager UUID.

    FrameT is an opaque adapter-specific frame payload, not a YOLO/OpenCV type
    in this domain. Initialization uses an externally supplied ROI. track returns
    a result with the same target ID and frame metadata; it never updates Target
    Manager or Mission State Machine. Adapters should reject use before initialize.
    No camera acquisition, lifecycle transitions, or command execution is implied.
    """

    def initialize(self, *, target_id: UUID, frame: FrameT, frame_info: FrameInfo,
                   bounding_box: BoundingBox) -> None:
        ...

    def track(self, *, target_id: UUID, frame: FrameT,
              frame_info: FrameInfo) -> TrackingObservation:
        ...
