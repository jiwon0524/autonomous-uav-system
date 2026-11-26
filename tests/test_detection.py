"""Contract fixtures only: no models, downloads, camera or CV installation."""
import ast
from dataclasses import FrozenInstanceError
from pathlib import Path
from types import SimpleNamespace
import subprocess
import sys
import unittest
from unittest.mock import Mock, patch

from src.autonomy.detection import Detection
from src.autonomy.targets import TargetType
from src.autonomy.tracking import BoundingBox, FrameInfo, ImageSize
from src.autonomy.vision.yolo import DetectionConversionError, DetectorError, YoloDetector
from src.autonomy.vision.webcam_demo import main

FRAME = FrameInfo(7, image_size=ImageSize(100, 80))


def tensor(values):
    return SimpleNamespace(cpu=lambda: SimpleNamespace(tolist=lambda: values))


def result(coords=(), scores=(), classes=()):
    return SimpleNamespace(orig_shape=(80, 100), boxes=SimpleNamespace(
        xyxy=tensor(list(coords)), conf=tensor(list(scores)), cls=tensor(list(classes))))


def fixture_model():
    return Mock(task="detect", names={4: "person", 9: "car", 12: "chair"})


class DetectionTests(unittest.TestCase):
    def setUp(self):
        self.model = fixture_model()
        self.loader = patch('src.autonomy.vision.yolo._load_model', return_value=self.model)
        self.loader.start()
        self.addCleanup(self.loader.stop)
        self.detector = YoloDetector()

    def test_model_creation_reuses_box_and_frame(self):
        box = BoundingBox(1, 2, 30, 40)
        detection = Detection(TargetType.PERSON, "person", box, 0.9, FRAME, 4)
        self.assertIs(detection.bounding_box, box)
        self.assertIs(detection.frame, FRAME)
        self.assertEqual(detection.class_id, 4)
        self.assertFalse(hasattr(detection, 'target_id'))

    def test_confidence_validation(self):
        for score in (0, 0.2, 1):
            self.assertEqual(Detection(TargetType.OBJECT, 'x', BoundingBox(0, 0, 1, 1), score, FRAME).confidence, score)
        for score in (-1, 1.1, float('nan'), float('inf'), True, '0.5', 10**1000):
            with self.subTest(score_type=type(score)), self.assertRaises((ValueError, TypeError)):
                Detection(TargetType.OBJECT, 'x', BoundingBox(0, 0, 1, 1), score, FRAME)

    def test_class_type_and_optional_id(self):
        item = Detection(TargetType.VEHICLE, 'car', BoundingBox(0, 0, 1, 1), 1, FRAME)
        self.assertEqual((item.object_type, item.class_name, item.class_id), (TargetType.VEHICLE, 'car', None))
        for kwargs in ({'object_type': 'person'}, {'class_name': ''}, {'class_id': True},
                       {'class_id': -1}, {'frame': object()}, {'bounding_box': (0, 0, 1, 1)}):
            args = dict(object_type=TargetType.PERSON, class_name='person', class_id=0,
                        bounding_box=BoundingBox(0, 0, 1, 1), confidence=0.8, frame=FRAME)
            args.update(kwargs)
            with self.assertRaises((TypeError, ValueError)):
                Detection(**args)

    def test_immutable(self):
        item = Detection(TargetType.OBJECT, 'chair', BoundingBox(0, 0, 1, 1), 1, FRAME)
        with self.assertRaises(FrozenInstanceError):
            item.confidence = 0

    def test_image_bounds(self):
        with self.assertRaises(ValueError):
            Detection(TargetType.OBJECT, 'x', BoundingBox(0, 0, 101, 2), 1, FRAME)

    def test_conversion_plain_values(self):
        converted = self.detector.convert_result(result([[1, 2, 30, 40]], [0.9], [4.0]), FRAME)
        self.assertIsInstance(converted, tuple)
        item = converted[0]
        self.assertEqual(item, Detection(TargetType.PERSON, 'person', BoundingBox(1, 2, 30, 40), 0.9, FRAME, 4))
        self.assertFalse(hasattr(item, 'boxes'))

    def test_person_filter_uses_model_names_not_class_zero(self):
        items = self.detector.convert_result(result([[0, 0, 10, 10]]*2, [0.9]*2, [9, 4]), FRAME)
        self.assertEqual([item.class_id for item in items], [4])

    def test_confidence_filter_and_boundary(self):
        items = self.detector.convert_result(result([[0, 0, 10, 10]]*3, [0.2, 0.25, 0.9], [4]*3), FRAME)
        self.assertEqual([i.confidence for i in items], [0.25, 0.9])

    def test_all_labels_and_multiple_results(self):
        detector = YoloDetector(class_names=None)
        items = detector.convert_result(result([[0, 0, 10, 10]]*3, [0.9]*3, [4, 9, 12]), FRAME)
        self.assertEqual([i.object_type for i in items], [TargetType.PERSON, TargetType.VEHICLE, TargetType.OBJECT])

    def test_custom_label_filter(self):
        detector = YoloDetector(class_names=['car'])
        self.assertEqual(len(detector.convert_result(result([[0, 0, 1, 1]], [1], [9]), FRAME)), 1)

    def test_empty_detections(self):
        self.assertEqual(self.detector.convert_result(result(), FRAME), ())

    def test_invalid_boxes_reject_entire_frame(self):
        for box in ([0, 0, 0, 1], [-1, 0, 10, 10], [0, 0, 101, 10],
                    [0, 0, float('nan'), 1], [0, 0, 1]):
            with self.subTest(box=box), self.assertRaises(DetectionConversionError):
                self.detector.convert_result(result([[0, 0, 1, 1], box], [1, 1], [4, 4]), FRAME)
        # Bad frame cannot leave cached/partial detections in the adapter.
        self.assertEqual(self.detector.convert_result(result(), FRAME), ())

    def test_malformed_arrays_scores_and_ids(self):
        for value in (result([[0, 0, 1, 1]], [], [4]), result([[0, 0, 1, 1]], [1.1], [4]),
                      result([[0, 0, 1, 1]], [1], [4.5]), result([[0, 0, 1, 1]], [1], [999]),
                      SimpleNamespace(orig_shape=(80, 100), boxes=None)):
            with self.assertRaises(DetectionConversionError):
                self.detector.convert_result(value, FRAME)

    def test_frame_shape_mismatch(self):
        value = result()
        value.orig_shape = (90, 100)
        with self.assertRaises(DetectionConversionError):
            self.detector.convert_result(value, FRAME)

    def test_predict_single_image_no_tracking_or_saving(self):
        image = SimpleNamespace(shape=(80, 100, 3))
        self.model.predict.return_value = [result()]
        self.assertEqual(self.detector.detect(image, FRAME), ())
        self.model.predict.assert_called_once_with(source=image, conf=0.25, classes=[4], device='cpu',
                                                   verbose=False, save=False, stream=False)
        self.model.track.assert_not_called()

    def test_inference_error_and_invalid_input(self):
        self.model.predict.side_effect = RuntimeError('inference unavailable')
        with self.assertRaisesRegex(DetectorError, 'inference failed'):
            self.detector.detect(SimpleNamespace(shape=(80, 100, 3)), FRAME)
        for image in (None, 'camera-url', SimpleNamespace(shape=(1, 1)), SimpleNamespace(shape=(81, 100, 3))):
            with self.assertRaises(DetectorError):
                self.detector.detect(image, FRAME)

    def test_model_load_and_label_errors(self):
        with patch('src.autonomy.vision.yolo._load_model', side_effect=RuntimeError('missing weights')):
            with self.assertRaisesRegex(DetectorError, 'Could not load'):
                YoloDetector('missing.pt')
        with self.assertRaisesRegex(DetectorError, 'requested labels'):
            YoloDetector(class_names=['unknown'])
        self.model.task = 'classify'
        with self.assertRaisesRegex(DetectorError, 'object detection'):
            YoloDetector()

    def test_invalid_threshold_and_labels(self):
        for score in (-1, 2, float('nan'), True):
            with self.assertRaises((ValueError, TypeError)):
                YoloDetector(confidence=score)
        for labels in ('person', [], ['']):
            with self.assertRaises((ValueError, TypeError)):
                YoloDetector(class_names=labels)

    def test_domain_does_not_import_cv(self):
        path = Path(__file__).parents[1] / 'src/autonomy/detection/models.py'
        tree = ast.parse(path.read_text(encoding='utf-8'))
        imports = [n.module for n in ast.walk(tree) if isinstance(n, ast.ImportFrom)]
        self.assertFalse(any(any(s in (m or '') for s in ('ultralytics', 'cv2', 'vision')) for m in imports))
        # Fresh interpreter rejects optional imports, proving tests need no CV install.
        code = """import sys
class BlockCV:
    def find_spec(self, fullname, *args):
        if fullname.split('.')[0] in {'ultralytics', 'cv2', 'torch', 'numpy'}:
            raise AssertionError(fullname)
sys.meta_path.insert(0, BlockCV())
from src.autonomy.detection import Detection
from src.autonomy.vision.yolo import YoloDetector
"""
        completed = subprocess.run([sys.executable, '-B', '-c', code], capture_output=True, text=True,
                                   cwd=Path(__file__).parents[1])
        self.assertEqual(completed.returncode, 0, completed.stderr)

    def test_demo_setup_errors_no_camera(self):
        with patch('sys.stderr'), patch('src.autonomy.vision.webcam_demo.YoloDetector', side_effect=DetectorError('bad model')):
            self.assertEqual(main([]), 1)

    def test_demo_camera_errors_release_resources(self):
        for opened in (False, True):
            cv = Mock()
            cv.error = type('CVError', (Exception,), {})
            camera = cv.VideoCapture.return_value
            camera.isOpened.return_value = opened
            camera.read.return_value = (False, None)
            with patch.dict(sys.modules, {'cv2': cv}), patch('sys.stderr'):
                self.assertEqual(main([]), 1)
            camera.release.assert_called_once()
            cv.destroyAllWindows.assert_called_once()

    def test_demo_recovers_then_draws_current_detection(self):
        cv = Mock()
        cv.error = type('CVError', (Exception,), {})
        image = SimpleNamespace(shape=(80, 100, 3))
        cv.VideoCapture.return_value.read.return_value = (True, image)
        cv.waitKey.side_effect = [0, ord('q')]
        cv.getWindowProperty.return_value = 1
        detection = Detection(TargetType.PERSON, 'person', BoundingBox(0, 0, 10, 10), 0.9, FRAME)
        with patch.dict(sys.modules, {'cv2': cv}), patch('src.autonomy.vision.webcam_demo.YoloDetector') as factory, patch('sys.stderr'):
            factory.return_value.detect.side_effect = [DetectorError('transient'), (detection,)]
            self.assertEqual(main([]), 0)
        cv.rectangle.assert_called_once_with(image, (0, 0), (9, 9), (0, 160, 0), 2)
        cv.VideoCapture.return_value.release.assert_called_once()
        cv.destroyAllWindows.assert_called_once()

    def test_demo_repeated_errors_stop_and_cleanup(self):
        cv = Mock()
        cv.error = type('CVError', (Exception,), {})
        cv.VideoCapture.return_value.read.return_value = (True, SimpleNamespace(shape=(80, 100, 3)))
        cv.waitKey.return_value = 0
        cv.getWindowProperty.return_value = 1
        with patch.dict(sys.modules, {'cv2': cv}), patch('src.autonomy.vision.webcam_demo.YoloDetector') as factory, patch('sys.stderr'):
            factory.return_value.detect.side_effect = DetectorError('persistent')
            self.assertEqual(main([]), 1)
            self.assertEqual(factory.return_value.detect.call_count, 5)
        cv.VideoCapture.return_value.release.assert_called_once()
        cv.destroyAllWindows.assert_called_once()

    def test_demo_exit_keys_and_frame_error_recovery(self):
        for key in (ord('q'), 27):
            cv = Mock()
            cv.error = type('CVError', (Exception,), {})
            cv.VideoCapture.return_value.read.return_value = (True, SimpleNamespace(shape=(80, 100, 3)))
            cv.waitKey.return_value = key
            with patch.dict(sys.modules, {'cv2': cv}), patch('src.autonomy.vision.webcam_demo.YoloDetector') as factory, patch('sys.stderr'):
                factory.return_value.detect.side_effect = DetectorError('one bad frame')
                self.assertEqual(main([]), 0)
            cv.VideoCapture.return_value.release.assert_called_once()
            cv.destroyAllWindows.assert_called_once()


if __name__ == '__main__':
    unittest.main()
