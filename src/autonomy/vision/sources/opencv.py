"""Optional OpenCV adapter for device indices and user-supplied network URLs."""
from collections.abc import Callable
from datetime import datetime, timezone
from urllib.parse import urlsplit

from ...tracking import FrameInfo, ImageSize
from .models import CameraFrame
from .protocols import CameraSourceError, FrameReadError, SourceClosedError, SourceOpenError


def _capture(source):
    import cv2
    return cv2.VideoCapture(source)


def _utc_now():
    return datetime.now(timezone.utc)


class OpenCVSource:
    """Exactly one device index or network URL; construction opens no device.

    OpenCV's selected backend decides supported codecs/protocols and blocking
    behavior. No automatic reconnect or additional capture thread is created.
    Sequence counts successful reads over this object's lifetime, even across
    close/open. A new source instance starts a new sequence at zero.
    clock is injectable and describes host receive time, not sensor exposure.
    """
    def __init__(self, *, camera_index: int | None = None, url: str | None = None,
                 source_id: str | None = None, clock: Callable[[], datetime] = _utc_now):
        if (camera_index is None) == (url is None):
            raise ValueError("provide exactly one of camera_index or url")
        if camera_index is not None:
            if type(camera_index) is not int or camera_index < 0:
                raise ValueError("camera_index must be a nonnegative integer, not bool")
            self._input = camera_index
            self._description = f"camera index {camera_index}"
            default_id = f"opencv-camera:{camera_index}"
        else:
            if not isinstance(url, str) or not url or any(c.isspace() for c in url):
                raise ValueError("url must be a nonblank network URL without whitespace")
            try:
                parts = urlsplit(url)
                if not parts.scheme or not parts.hostname:
                    raise ValueError()
                _ = parts.port  # Reject invalid ports without exposing URL credentials.
            except ValueError:
                raise ValueError("url must contain a valid scheme, host and optional port") from None
            self._input = url
            self._description = "network video source"
            default_id = "opencv-network"
        if source_id is not None and (not isinstance(source_id, str) or not source_id.strip()):
            raise ValueError("source_id must be a nonblank string")
        if not callable(clock):
            raise TypeError("clock must be callable")
        self._source_id = source_id if source_id is not None else default_id
        self._clock = clock
        self._capture = None
        self._sequence = 0

    @property
    def is_open(self) -> bool:
        try:
            return self._capture is not None and bool(self._capture.isOpened())
        except Exception:
            raise CameraSourceError("Cannot query camera source status") from None

    def open(self) -> None:
        if self.is_open:
            return
        self.close()
        try:
            self._capture = _capture(self._input)
            if not self.is_open:
                raise SourceOpenError("capture backend did not open")
        except Exception:
            self.close()
            raise SourceOpenError(f"Cannot open {self._description}; check device, URL, permissions and OpenCV backend") from None

    def read(self) -> CameraFrame:
        if self._capture is None:
            raise SourceClosedError("Camera source is closed; call open before read")
        try:
            if not self.is_open:
                raise FrameReadError("capture backend is no longer open")
            ok, pixels = self._capture.read()
            if not ok or pixels is None:
                raise FrameReadError("backend returned no frame")
            shape = getattr(pixels, "shape", ())
            if len(shape) != 3 or shape[2] != 3 or shape[0] <= 0 or shape[1] <= 0:
                raise FrameReadError("expected a nonempty HxWx3 BGR image")
            received_at = self._clock()
            if not isinstance(received_at, datetime):
                raise FrameReadError("clock must return a timezone-aware datetime")
            info = FrameInfo(self._sequence, received_at,
                             ImageSize(int(shape[1]), int(shape[0])), self._source_id)
            frame = CameraFrame(pixels, info)
        except Exception:
            self.close()
            raise FrameReadError(f"Frame read failed for {self._description}; source closed") from None
        self._sequence += 1
        return frame

    def close(self) -> None:
        capture, self._capture = self._capture, None
        if capture is not None:
            try:
                capture.release()
            except Exception:
                raise CameraSourceError("Camera source resource release failed") from None
