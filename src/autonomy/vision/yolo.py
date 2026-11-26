"""Ultralytics-specific boundary. Only plain domain values leave this adapter."""
from math import isfinite

from ..detection import Detection
from ..detection.models import confidence_value
from ..targets import TargetType
from ..tracking import BoundingBox, FrameInfo

DEFAULT_MODEL = "yolo11n.pt"
VEHICLE_LABELS = frozenset({"bicycle", "car", "motorcycle", "bus", "truck", "train", "boat", "airplane"})


class DetectorError(RuntimeError):
    pass


class DetectionConversionError(DetectorError):
    pass


def _load_model(model):
    from ultralytics import YOLO
    return YOLO(model, task="detect")


class YoloDetector:
    """One image per call; no acquisition, matching, tracking or Target UUIDs.

    class_names=None includes all model labels. Default filters the model's
    actual name table for person, rather than assuming class 0 for every model.
    """
    def __init__(self, model=DEFAULT_MODEL, *, confidence=0.25,
                 class_names=("person",), device="cpu"):
        self.confidence = confidence_value(confidence)
        if class_names is not None:
            if isinstance(class_names, str):
                raise TypeError("class_names must be a collection, not a string")
            class_names = tuple(class_names)
            if not class_names or any(not isinstance(n, str) or not n.strip() for n in class_names):
                raise ValueError("class_names must contain nonblank labels")
            class_names = frozenset(n.strip().lower() for n in class_names)
        self.class_names = class_names
        self.device = device
        try:
            self._model = _load_model(model)
            if getattr(self._model, "task", None) != "detect":
                raise ValueError("model must support object detection")
            names = self._model.names
            self._names = dict(names) if isinstance(names, dict) else dict(enumerate(names))
            available = {name.lower() for name in self._names.values()}
            if class_names is not None and not class_names <= available:
                raise ValueError(f"requested labels not in model: {sorted(class_names - available)}")
            self._class_ids = (None if class_names is None else
                               [i for i, name in self._names.items() if name.lower() in class_names])
        except Exception as exc:
            raise DetectorError(f"Could not load detection model {model!r}: {exc}") from exc

    def detect(self, frame, frame_info: FrameInfo) -> tuple[Detection, ...]:
        if not isinstance(frame_info, FrameInfo):
            raise TypeError("frame_info must be a FrameInfo")
        # Acquisition belongs to the caller. Accept a single decoded image only.
        shape = getattr(frame, "shape", ())
        if len(shape) != 3 or shape[2] != 3 or shape[0] <= 0 or shape[1] <= 0:
            raise DetectorError("frame must be a nonempty HxWx3 BGR image")
        size = frame_info.image_size
        if size and (size.width != shape[1] or size.height != shape[0]):
            raise DetectorError("frame dimensions do not match FrameInfo")
        try:
            results = self._model.predict(source=frame, conf=self.confidence,
                                          classes=self._class_ids, device=self.device,
                                          verbose=False, save=False, stream=False)
        except Exception as exc:
            raise DetectorError(f"YOLO inference failed: {exc}") from exc
        if len(results) != 1:
            raise DetectorError("expected one result for one image")
        return self.convert_result(results[0], frame_info)

    def convert_result(self, result, frame_info: FrameInfo) -> tuple[Detection, ...]:
        """Reject the entire malformed frame; never silently expose partial results."""
        if not isinstance(frame_info, FrameInfo):
            raise TypeError("frame_info must be a FrameInfo")
        try:
            size = frame_info.image_size
            if size and tuple(result.orig_shape) != (size.height, size.width):
                raise ValueError("result dimensions do not match FrameInfo")
            boxes = result.boxes
            if boxes is None:
                raise ValueError("result has no detection boxes (wrong model/task)")
            coordinates = boxes.xyxy.cpu().tolist()
            scores = boxes.conf.cpu().tolist()
            classes = boxes.cls.cpu().tolist()
            if not len(coordinates) == len(scores) == len(classes):
                raise ValueError("inconsistent detection array lengths")
            detections = []
            for coords, score, raw_id in zip(coordinates, scores, classes):
                if isinstance(raw_id, bool) or not isfinite(raw_id) or raw_id < 0 or int(raw_id) != raw_id:
                    raise ValueError("invalid detector class ID")
                class_id = int(raw_id)
                name = self._names[class_id]
                label = name.lower()
                score = confidence_value(score)
                # Validate geometry even when a row would subsequently be filtered.
                object_type = (TargetType.PERSON if label == "person" else
                               TargetType.VEHICLE if label in VEHICLE_LABELS else TargetType.OBJECT)
                detection = Detection(object_type, name, BoundingBox(*coords), score, frame_info, class_id)
                if score >= self.confidence and (self.class_names is None or label in self.class_names):
                    detections.append(detection)
            return tuple(detections)
        except (AttributeError, KeyError, TypeError, ValueError, OverflowError) as exc:
            raise DetectionConversionError(f"Invalid YOLO detection result: {exc}") from exc
