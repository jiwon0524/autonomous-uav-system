"""Isolated contract fixtures only; no runtime detector, camera or fake loop."""
import ast
from dataclasses import FrozenInstanceError
from datetime import datetime, timezone
from pathlib import Path
import unittest
from uuid import UUID

from src.autonomy.targets import ExternalTargetId, Observation, TargetManager, TargetType
from src.autonomy.tracking import BoundingBox, FrameInfo, ImageSize, Tracker, TrackingObservation

TARGET_ID = UUID(int=1)


class TrackingTests(unittest.TestCase):
    def test_valid_box_and_continuous_pixel_edges(self):
        box = BoundingBox(10, 20.5, 40, 50.5)
        self.assertEqual((box.x1, box.y1, box.x2, box.y2), (10, 20.5, 40, 50.5))

    def test_width_and_height(self):
        box = BoundingBox(10, 20, 50, 80)
        self.assertEqual(box.width, 40)
        self.assertEqual(box.height, 60)

    def test_center(self):
        self.assertEqual(BoundingBox(10, 20, 50, 80).center, (30, 50))
        self.assertEqual(BoundingBox(0, 0, 1, 1).center, (0.5, 0.5))

    def test_invalid_boxes_rejected(self):
        for values in [(0, 0, 0, 1), (0, 0, 1, 0), (4, 0, 2, 1), (0, 4, 1, 2),
                       (-1, 0, 1, 1), (0, -1, 1, 1), (0, 0, float('inf'), 1),
                       (float('nan'), 0, 1, 1), (0, 0, 1, float('-inf'))]:
            with self.subTest(values=values):
                with self.assertRaises(ValueError):
                    BoundingBox(*values)

    def test_nonnumeric_box_values_rejected(self):
        for value in ('1', True, None, object()):
            with self.assertRaises(TypeError):
                BoundingBox(value, 0, 2, 2)

    def test_observed_result_and_existing_target_id_preserved(self):
        manager = TargetManager()
        target = manager.register_target(TargetType.PERSON)
        box = BoundingBox(10, 20, 30, 40)
        result = TrackingObservation(target.id, FrameInfo(1), True, box)
        self.assertEqual(result.target_id, target.id)
        self.assertTrue(result.observed)
        self.assertTrue(result.success)
        self.assertEqual(result.center, (20, 30))
        self.assertIsNone(result.confidence)
        self.assertEqual(len(manager.list_targets()), 1)

    def test_failed_result_has_no_geometry_or_score(self):
        result = TrackingObservation(TARGET_ID, FrameInfo(2), False)
        self.assertFalse(result.observed)
        self.assertFalse(result.success)
        self.assertIsNone(result.bounding_box)
        self.assertIsNone(result.center)
        self.assertIsNone(result.confidence)

    def test_inconsistent_success_failure_rejected(self):
        with self.assertRaises(ValueError):
            TrackingObservation(TARGET_ID, FrameInfo(1), True)
        with self.assertRaises(ValueError):
            TrackingObservation(TARGET_ID, FrameInfo(1), False, BoundingBox(0, 0, 1, 1))
        with self.assertRaises(ValueError):
            TrackingObservation(TARGET_ID, FrameInfo(1), False, confidence=0)

    def test_optional_confidence(self):
        for score in (None, 0, 0.5, 1):
            result = TrackingObservation(TARGET_ID, FrameInfo(1), True, BoundingBox(0, 0, 1, 1), score)
            self.assertEqual(result.confidence, score)

    def test_invalid_confidence(self):
        for score in (-0.1, 1.1, float('nan'), float('inf')):
            with self.assertRaises(ValueError):
                TrackingObservation(TARGET_ID, FrameInfo(1), True, BoundingBox(0, 0, 1, 1), score)
        for score in (True, '0.5'):
            with self.assertRaises(TypeError):
                TrackingObservation(TARGET_ID, FrameInfo(1), True, BoundingBox(0, 0, 1, 1), score)

    def test_frame_metadata_reuses_existing_observation_contract(self):
        at = datetime(2026, 10, 31, tzinfo=timezone.utc)
        frame = FrameInfo(12, at, ImageSize(640, 480), 'opaque-frame')
        self.assertIsInstance(frame, Observation)
        self.assertEqual(frame.sequence, 12)
        self.assertEqual(frame.observed_at, at)
        self.assertEqual(frame.frame_id, 'opaque-frame')
        result = TrackingObservation(TARGET_ID, frame, True, BoundingBox(0, 0, 640, 480))
        self.assertEqual(result.frame, frame)

    def test_invalid_frame_metadata(self):
        for sequence in (-1,):
            with self.assertRaises(ValueError):
                FrameInfo(sequence)
        for sequence in (True, 1.5, '1'):
            with self.assertRaises(TypeError):
                FrameInfo(sequence)
        with self.assertRaises(ValueError):
            FrameInfo(1, datetime(2026, 10, 31))
        with self.assertRaises(TypeError):
            FrameInfo(1, image_size=(640, 480))
        with self.assertRaises(ValueError):
            FrameInfo(1, frame_id='')

    def test_resolution_is_optional_but_enforced_when_supplied(self):
        TrackingObservation(TARGET_ID, FrameInfo(1), True, BoundingBox(0, 0, 100, 100))
        with self.assertRaises(ValueError):
            TrackingObservation(TARGET_ID, FrameInfo(1, image_size=ImageSize(10, 10)), True, BoundingBox(0, 0, 11, 10))
        for width, height in [(0, 1), (1, -1)]:
            with self.assertRaises(ValueError):
                ImageSize(width, height)
        for width in (True, 1.5):
            with self.assertRaises(TypeError):
                ImageSize(width, 1)

    def test_external_identity_reuses_target_manager_value(self):
        external = ExternalTargetId('adapter-session', 'opaque-id')
        result = TrackingObservation(TARGET_ID, FrameInfo(1), False, external_id=external)
        self.assertEqual(result.external_id, external)
        self.assertEqual(result.target_id, TARGET_ID)

    def test_invalid_observation_field_types(self):
        for kwargs in [dict(target_id='target-1', frame=FrameInfo(1), observed=False),
                       dict(target_id=TARGET_ID, frame=Observation(1), observed=False),
                       dict(target_id=TARGET_ID, frame=FrameInfo(1), observed=1),
                       dict(target_id=TARGET_ID, frame=FrameInfo(1), observed=True, bounding_box=(0, 0, 1, 1)),
                       dict(target_id=TARGET_ID, frame=FrameInfo(1), observed=False, external_id='1')]:
            with self.subTest(kwargs=kwargs):
                with self.assertRaises(TypeError):
                    TrackingObservation(**kwargs)

    def test_nested_models_are_immutable(self):
        box = BoundingBox(0, 0, 2, 2)
        frame = FrameInfo(1, image_size=ImageSize(4, 4))
        result = TrackingObservation(TARGET_ID, frame, True, box, 0.5, ExternalTargetId('a', '1'))
        for obj, name, value in [(box, 'x1', 1), (frame, 'sequence', 2),
                                  (frame.image_size, 'width', 8), (result, 'target_id', UUID(int=2)),
                                  (result.external_id, 'value', '2')]:
            with self.assertRaises(FrozenInstanceError):
                setattr(obj, name, value)

    def test_failed_observation_does_not_change_target_manager_or_mission(self):
        from src.autonomy.core import MissionStateMachine, MissionState
        manager, mission = TargetManager(), MissionStateMachine()
        target = manager.register_target(TargetType.OBJECT)
        manager.select_target(target.id)
        manager.start_tracking(target.id)
        before = manager.get_target(target.id)
        TrackingObservation(target.id, FrameInfo(1), False)
        self.assertEqual(manager.get_target(target.id), before)
        self.assertEqual(mission.state, MissionState.IDLE)

    def test_protocol_accepts_library_independent_contract_fixture(self):
        class ContractFixture:
            def initialize(self, *, target_id, frame, frame_info, bounding_box):
                pass

            def track(self, *, target_id, frame, frame_info):
                return TrackingObservation(target_id, frame_info, False)

        fixture = ContractFixture()
        self.assertIsInstance(fixture, Tracker)
        self.assertNotIsInstance(object(), Tracker)
        result = fixture.track(target_id=TARGET_ID, frame=object(), frame_info=FrameInfo(8))
        self.assertEqual(result.target_id, TARGET_ID)
        self.assertEqual(result.frame.sequence, 8)

    def test_domain_imports_no_cv_ros_or_mission_implementation(self):
        directory = Path(__file__).resolve().parents[1] / 'src/autonomy/tracking'
        for path in directory.glob('*.py'):
            tree = ast.parse(path.read_text(encoding='utf-8'))
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    modules = [name.name for name in node.names]
                elif isinstance(node, ast.ImportFrom):
                    modules = [node.module or '']
                else:
                    continue
                for module in modules:
                    self.assertFalse(any(word in module for word in ('cv2', 'ultralytics', 'numpy', 'rclpy', 'px4_msgs', 'mission_state_machine')))


if __name__ == '__main__':
    unittest.main()
