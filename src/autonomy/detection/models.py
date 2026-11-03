"""Detection values only; no inference, target selection or tracking state."""
from dataclasses import dataclass
from math import isfinite

from ..targets import TargetType
from ..tracking import BoundingBox, FrameInfo


def confidence_value(value: float) -> float:
    if type(value) not in (int, float):
        raise TypeError("confidence must be a number, not bool")
    try:
        score = float(value)
    except OverflowError:
        raise ValueError("confidence must be finite and in [0, 1]") from None
    if not isfinite(score) or not 0 <= score <= 1:
        raise ValueError("confidence must be finite and in [0, 1]")
    return score


@dataclass(frozen=True)
class Detection:
    object_type: TargetType
    class_name: str
    bounding_box: BoundingBox
    confidence: float
    frame: FrameInfo
    class_id: int | None = None

    def __post_init__(self):
        if not isinstance(self.object_type, TargetType):
            raise TypeError("object_type must be a TargetType")
        if not isinstance(self.class_name, str) or not self.class_name.strip():
            raise ValueError("class_name must be a nonblank string")
        if not isinstance(self.bounding_box, BoundingBox):
            raise TypeError("bounding_box must be a BoundingBox")
        if not isinstance(self.frame, FrameInfo):
            raise TypeError("frame must be a FrameInfo")
        if self.class_id is not None and (type(self.class_id) is not int or self.class_id < 0):
            raise ValueError("class_id must be a nonnegative integer or None")
        object.__setattr__(self, "confidence", confidence_value(self.confidence))
        size = self.frame.image_size
        if size and (self.bounding_box.x2 > size.width or self.bounding_box.y2 > size.height):
            raise ValueError("detection bounding box exceeds image size")
