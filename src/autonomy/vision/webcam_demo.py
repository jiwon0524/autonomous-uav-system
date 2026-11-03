"""Manual camera/network video check; never connected to Dashboard or UAV control."""
import argparse
from math import ceil
import sys
from time import perf_counter

from .sources import CameraSource, CameraSourceError, OpenCVSource
from .yolo import DEFAULT_MODEL, DetectorError, YoloDetector


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", choices=("camera", "url"), default="camera")
    parser.add_argument("--camera", type=int, default=0)
    parser.add_argument("--url", help="OpenCV-compatible network stream URL")
    parser.add_argument("--model", default=DEFAULT_MODEL)
    parser.add_argument("--confidence", type=float, default=0.25)
    parser.add_argument("--classes", nargs="+", default=["person"], help="model labels, or all")
    parser.add_argument("--device", default="cpu", help="cpu by default; optional Ultralytics device")
    args = parser.parse_args(argv)
    try:
        if args.source == "url":
            if args.url is None:
                parser.error("--source url requires --url")
            source: CameraSource = OpenCVSource(url=args.url)
        else:
            if args.url is not None:
                parser.error("--url requires --source url")
            source = OpenCVSource(camera_index=args.camera)
    except ValueError as exc:
        parser.error(str(exc))
    try:
        detector = YoloDetector(args.model, confidence=args.confidence,
                                class_names=None if args.classes == ["all"] else args.classes,
                                device=args.device)
    except (DetectorError, ValueError, TypeError) as exc:
        print(f"Detector setup error: {exc}", file=sys.stderr)
        return 1
    try:
        import cv2
    except ImportError as exc:
        print(f"Install requirements-vision.txt (OpenCV unavailable): {exc}", file=sys.stderr)
        return 1
    exit_code = 0
    try:
        source.open()
        failures = 0
        while True:
            frame = source.read()
            image, info = frame.pixels, frame.info
            started = perf_counter()
            try:
                detections = detector.detect(image, info)
                failures = 0
                for item in detections:
                    box = item.bounding_box
                    # Exclusive domain edges -> inclusive display pixel indices.
                    p1 = (int(box.x1), int(box.y1))
                    p2 = (min(image.shape[1]-1, ceil(box.x2)-1), min(image.shape[0]-1, ceil(box.y2)-1))
                    cv2.rectangle(image, p1, p2, (0, 160, 0), 2)
                    cv2.putText(image, f"{item.class_name} {item.confidence:.2f}",
                                (p1[0], max(18, p1[1]-6)), cv2.FONT_HERSHEY_SIMPLEX, 0.6, (0, 120, 0), 2)
                message = f"Processing {(perf_counter()-started)*1000:.0f} ms | q / ESC to exit"
            except DetectorError as exc:
                failures += 1
                print(f"Frame {info.sequence}: {exc}", file=sys.stderr)
                message = "Detection error - see terminal (q / ESC to exit)"
                if failures >= 5:
                    raise RuntimeError("Stopping after 5 consecutive detection failures") from exc
            cv2.putText(image, message, (10, 25), cv2.FONT_HERSHEY_SIMPLEX, 0.55, (0, 0, 180), 1)
            cv2.imshow("UAV Detection - development only", image)
            if cv2.waitKey(1) & 0xFF in (ord('q'), 27):
                break
            if cv2.getWindowProperty("UAV Detection - development only", cv2.WND_PROP_VISIBLE) < 1:
                break
    except KeyboardInterrupt:
        pass
    except (RuntimeError, cv2.error) as exc:
        print(f"Camera demo error: {exc}", file=sys.stderr)
        exit_code = 1
    finally:
        try:
            source.close()
        except CameraSourceError as exc:
            print(f"Camera cleanup error: {exc}", file=sys.stderr)
            exit_code = 1
        try:
            cv2.destroyAllWindows()
        except cv2.error as exc:
            print(f"Window cleanup error: {exc}", file=sys.stderr)
            exit_code = 1
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
