"""A frame envelope; pixels are borrowed, not copied or frozen."""
from dataclasses import dataclass
from typing import Generic, TypeVar

from ...tracking import FrameInfo

PixelT = TypeVar("PixelT", covariant=True)


@dataclass(frozen=True)
class CameraFrame(Generic[PixelT]):
    """Immutable envelope only; callers must respect the adapter's buffer lifetime.

    FrameInfo carries sequence, image size, receive time and source identifier
    (frame_id). PixelT keeps transport/CV-specific types outside this contract.
    OpenCVSource provides BGR images owned by the caller after read; consumers
    sharing that image must coordinate in-place drawing/modification themselves.
    """
    pixels: PixelT
    info: FrameInfo

    def __post_init__(self):
        if self.pixels is None:
            raise ValueError("frame pixels must not be None")
        if not isinstance(self.info, FrameInfo):
            raise TypeError("info must be a FrameInfo")
