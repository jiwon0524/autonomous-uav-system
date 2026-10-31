"""Immutable image-space values. No tracker algorithm or target lifecycle logic."""
from dataclasses import dataclass
from math import isfinite
from uuid import UUID

from ..targets import ExternalTargetId, Observation


def _finite_number(name: str, value: int | float) -> float:
    if type(value) not in (int, float):
        raise TypeError(f"{name} must be an int or float, not bool")
    try:
        result = float(value)
    except OverflowError:
        raise ValueError(f"{name} must be finite") from None
    if not isfinite(result):
        raise ValueError(f"{name} must be finite")
    return result


@dataclass(frozen=True)
class BoundingBox:
    """Continuous pixel edges: top-left origin; x right, y down; x2/y2 exclusive.

    Coordinates are not normalized. No clipping or coordinate conversion occurs.
    Image bounds are checked by TrackingObservation when resolution is supplied.
    """
    x1: float
    y1: float
    x2: float
    y2: float

    def __post_init__(self):
        for name in ("x1", "y1", "x2", "y2"):
            object.__setattr__(self, name, _finite_number(name, getattr(self, name)))
        if self.x1 < 0 or self.y1 < 0:
            raise ValueError("top-left pixel coordinates must be nonnegative")
        if self.x2 <= self.x1 or self.y2 <= self.y1:
            raise ValueError("bounding box must have positive width and height")

    @property
    def width(self) -> float:
        return self.x2 - self.x1

    @property
    def height(self) -> float:
        return self.y2 - self.y1

    @property
    def center(self) -> tuple[float, float]:
        return (self.x1 + self.width / 2, self.y1 + self.height / 2)


@dataclass(frozen=True)
class ImageSize:
    width: int
    height: int

    def __post_init__(self):
        for name in ("width", "height"):
            value = getattr(self, name)
            if type(value) is not int:
                raise TypeError(f"{name} must be an integer")
            if value <= 0:
                raise ValueError(f"{name} must be positive")


@dataclass(frozen=True)
class FrameInfo(Observation):
    """Reuses Target Manager's sequence/time contract; no clock is read here.

    The adapter supplies sequence in one consistent stream/session. frame_id is
    optional source metadata, not a new target ID or a spatial coordinate frame.
    """
    image_size: ImageSize | None = None
    frame_id: str | None = None

    def __post_init__(self):
        super().__post_init__()
        if self.image_size is not None and not isinstance(self.image_size, ImageSize):
            raise TypeError("image_size must be an ImageSize or None")
        if self.frame_id is not None:
            if not isinstance(self.frame_id, str):
                raise TypeError("frame_id must be a string or None")
            if not self.frame_id.strip():
                raise ValueError("frame_id must not be blank")


@dataclass(frozen=True)
class TrackingObservation:
    """One result for an existing target ID and one frame, not a state machine.

    A failed observation cannot expose a stale box/score as a current detection.
    This value cannot verify target existence or real-world object identity.
    """
    target_id: UUID
    frame: FrameInfo
    observed: bool
    bounding_box: BoundingBox | None = None
    confidence: float | None = None
    external_id: ExternalTargetId | None = None

    def __post_init__(self):
        if not isinstance(self.target_id, UUID):
            raise TypeError("target_id must be the Target Manager UUID")
        if not isinstance(self.frame, FrameInfo):
            raise TypeError("frame must be a FrameInfo")
        if type(self.observed) is not bool:
            raise TypeError("observed must be a bool")
        if self.bounding_box is not None and not isinstance(self.bounding_box, BoundingBox):
            raise TypeError("bounding_box must be a BoundingBox or None")
        if self.external_id is not None and not isinstance(self.external_id, ExternalTargetId):
            raise TypeError("external_id must be an ExternalTargetId or None")
        if self.observed and self.bounding_box is None:
            raise ValueError("a successful observation requires a bounding box")
        if not self.observed and (self.bounding_box is not None or self.confidence is not None):
            raise ValueError("a failed observation must not carry a bounding box or confidence")
        if self.confidence is not None:
            score = _finite_number("confidence", self.confidence)
            if not 0 <= score <= 1:
                raise ValueError("confidence must be in [0, 1]")
            object.__setattr__(self, "confidence", score)
        if self.bounding_box is not None and self.frame.image_size is not None:
            size = self.frame.image_size
            if self.bounding_box.x2 > size.width or self.bounding_box.y2 > size.height:
                raise ValueError("bounding box exceeds the supplied image size")

    @property
    def success(self) -> bool:
        return self.observed

    @property
    def center(self) -> tuple[float, float] | None:
        return self.bounding_box.center if self.bounding_box is not None else None
