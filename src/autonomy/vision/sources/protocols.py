"""Transport-independent synchronous camera input boundary."""
from typing import Protocol, TypeVar, runtime_checkable

from .models import CameraFrame

PixelT = TypeVar("PixelT", covariant=True)


class CameraSourceError(RuntimeError):
    pass


class SourceOpenError(CameraSourceError):
    pass


class SourceClosedError(CameraSourceError):
    pass


class FrameReadError(CameraSourceError):
    pass


@runtime_checkable
class CameraSource(Protocol[PixelT]):
    """Sequential calls by a single owner; implementations own acquisition.

    read returns a new observation or raises, never a stale frame or None.
    close is idempotent; open on an open source is idempotent. No automatic
    reconnect, background acquisition, timestamp synchronization or inference.
    Future ROS/Gazebo adapters implement this contract without VideoCapture.
    """
    @property
    def is_open(self) -> bool:
        ...

    def open(self) -> None:
        ...

    def read(self) -> CameraFrame[PixelT]:
        ...

    def close(self) -> None:
        ...
