"""Test-only frame/capture fixtures: no CV imports, devices or network calls."""
from dataclasses import FrozenInstanceError
from datetime import datetime, timezone
from pathlib import Path
from types import SimpleNamespace
import ast
import subprocess
import sys
import unittest
from unittest.mock import Mock, patch

from src.autonomy.tracking import FrameInfo, ImageSize
from src.autonomy.vision.sources import (
    CameraFrame, CameraSource, CameraSourceError, FrameReadError,
    OpenCVSource, SourceClosedError, SourceOpenError,
)
from src.autonomy.vision.webcam_demo import main

AT = datetime(2026, 10, 31, tzinfo=timezone.utc)


class CameraSourceTests(unittest.TestCase):
    def setUp(self):
        self.capture = Mock()
        self.capture.isOpened.return_value = True
        self.pixels = SimpleNamespace(shape=(480, 640, 3))
        self.capture.read.return_value = (True, self.pixels)
        self.factory = patch('src.autonomy.vision.sources.opencv._capture', return_value=self.capture)
        self.create = self.factory.start()
        self.addCleanup(self.factory.stop)

    def source(self, **kwargs):
        return OpenCVSource(clock=lambda: AT, **kwargs)

    def test_protocol_accepts_non_opencv_source(self):
        class TransportFixture:
            is_open = False

            def open(self):
                self.is_open = True

            def read(self):
                return CameraFrame(object(), FrameInfo(0))

            def close(self):
                self.is_open = False
        source = TransportFixture()
        self.assertIsInstance(source, CameraSource)
        self.assertNotIsInstance(object(), CameraSource)
        source.open()
        self.assertIsInstance(source.read(), CameraFrame)
        source.close()
        self.assertFalse(source.is_open)

    def test_construction_does_not_open(self):
        source = self.source(camera_index=2)
        self.assertFalse(source.is_open)
        self.create.assert_not_called()

    def test_local_camera_index_forwarded(self):
        for index in (0, 1, 2):
            source = self.source(camera_index=index)
            source.open()
            self.create.assert_called_with(index)
            self.assertEqual(source.read().info.frame_id, f'opencv-camera:{index}')
            source.close()

    def test_network_url_forwarded_unchanged(self):
        for url in ('http://example.test/video', 'rtsp://example.test:8554/live',
                    'https://example.test/feed?token=sample', 'udp://example.test:5000'):
            source = self.source(url=url)
            source.open()
            self.create.assert_called_with(url)
            self.assertEqual(source.read().info.frame_id, 'opencv-network')
            source.close()

    def test_exclusive_input_required(self):
        for kwargs in ({}, {'camera_index': 0, 'url': 'http://example.test/live'}):
            with self.assertRaises(ValueError):
                self.source(**kwargs)

    def test_invalid_camera_indices(self):
        for value in (-1, True, '0', 1.5):
            with self.subTest(value=value), self.assertRaises(ValueError):
                self.source(camera_index=value)

    def test_invalid_urls(self):
        for value in ('', ' ', '0', 'camera.mp4', '/tmp/video', 'http://',
                      'http://example.test:invalid/live', 'http://example.test:99999/live',
                      'rtsp://bad host/live', 'file:///video.mp4', 'http://[broken', 123):
            with self.subTest(value=value), self.assertRaises(ValueError):
                self.source(url=value)

    def test_invalid_identifier_and_clock(self):
        for value in ('', ' ', 1):
            with self.assertRaises(ValueError):
                self.source(camera_index=0, source_id=value)
        with self.assertRaises(TypeError):
            OpenCVSource(camera_index=0, clock=AT)

    def test_frame_metadata_and_no_pixel_copy(self):
        source = self.source(camera_index=1, source_id='session-a:usb')
        source.open()
        frame = source.read()
        self.assertIs(frame.pixels, self.pixels)
        self.assertEqual(frame.info, FrameInfo(0, AT, ImageSize(640, 480), 'session-a:usb'))
        self.assertEqual(source.read().info.sequence, 1)
        self.assertTrue(source.is_open)
        source.close()

    def test_open_close_idempotent_and_reopen_preserves_sequence(self):
        source = self.source(camera_index=0)
        source.close()
        source.open()
        source.open()
        self.create.assert_called_once_with(0)
        self.assertEqual(source.read().info.sequence, 0)
        source.close()
        source.close()
        self.capture.release.assert_called_once()
        self.assertFalse(source.is_open)
        source.open()
        self.assertEqual(source.read().info.sequence, 1)
        source.close()

    def test_read_before_open_and_after_close(self):
        source = self.source(camera_index=0)
        with self.assertRaises(SourceClosedError):
            source.read()
        source.open()
        source.close()
        with self.assertRaises(SourceClosedError):
            source.read()
        self.capture.read.assert_not_called()

    def test_camera_and_network_open_failure_release(self):
        self.capture.isOpened.return_value = False
        for kwargs, expected in (({'camera_index': 2}, 'camera index 2'),
                                 ({'url': 'rtsp://example.test/live'}, 'network video source')):
            self.capture.release.reset_mock()
            source = self.source(**kwargs)
            with self.assertRaisesRegex(SourceOpenError, expected):
                source.open()
            self.capture.release.assert_called_once()
            self.assertFalse(source.is_open)
            source.close()
            self.capture.release.assert_called_once()

    def test_backend_open_exception_or_missing_dependency(self):
        for error in (RuntimeError('backend'), ImportError('cv2')):
            self.create.side_effect = error
            source = self.source(camera_index=0)
            with self.assertRaises(SourceOpenError):
                source.open()
            self.assertFalse(source.is_open)

    def test_frame_read_failure_closes_source(self):
        for output in ((False, None), (True, None)):
            self.capture.release.reset_mock()
            self.capture.read.return_value = output
            source = self.source(camera_index=0)
            source.open()
            with self.assertRaises(FrameReadError):
                source.read()
            self.assertFalse(source.is_open)
            self.capture.release.assert_called_once()
            with self.assertRaises(SourceClosedError):
                source.read()

    def test_frame_read_exception_closes_source(self):
        source = self.source(url='rtsp://example.test/live')
        source.open()
        self.capture.read.side_effect = RuntimeError('decoder error')
        with self.assertRaises(FrameReadError):
            source.read()
        self.capture.release.assert_called_once()
        self.assertFalse(source.is_open)

    def test_invalid_frame_shape_closes_source(self):
        for shape in ((), (0, 640, 3), (480, 640), (480, 640, 4)):
            self.capture.release.reset_mock()
            self.capture.read.return_value = (True, SimpleNamespace(shape=shape))
            source = self.source(camera_index=0)
            source.open()
            with self.assertRaises(FrameReadError):
                source.read()
            self.capture.release.assert_called_once()

    def test_invalid_timestamp_closes_source(self):
        for value in (None, datetime(2026, 10, 31), 'invalid'):
            self.capture.release.reset_mock()
            source = OpenCVSource(camera_index=0, clock=lambda: value)
            source.open()
            with self.assertRaises(FrameReadError):
                source.read()
            self.capture.release.assert_called_once()

    def test_backend_disconnect_closes_source(self):
        source = self.source(camera_index=0)
        source.open()
        self.capture.isOpened.return_value = False
        with self.assertRaises(FrameReadError):
            source.read()
        self.capture.release.assert_called_once()
        self.assertFalse(source.is_open)

    def test_release_error_reported_and_handle_detached(self):
        source = self.source(camera_index=0)
        source.open()
        self.capture.release.side_effect = RuntimeError('release error')
        with self.assertRaisesRegex(CameraSourceError, 'release failed'):
            source.close()
        self.assertFalse(source.is_open)
        source.close()
        self.capture.release.assert_called_once()

    def test_network_metadata_and_python_errors_do_not_expose_url(self):
        url = 'rtsp://user:secret@example.test/live?token=private'
        source = self.source(url=url)
        self.create.side_effect = RuntimeError(url)
        with self.assertRaises(SourceOpenError) as error:
            source.open()
        self.assertNotIn('secret', str(error.exception))
        self.assertNotIn('private', str(error.exception))

    def test_frame_envelope_is_frozen_without_freezing_pixels(self):
        frame = CameraFrame(self.pixels, FrameInfo(0))
        with self.assertRaises(FrozenInstanceError):
            frame.info = FrameInfo(1)
        self.pixels.marker = 'caller-owned'
        self.assertEqual(frame.pixels.marker, 'caller-owned')
        with self.assertRaises(ValueError):
            CameraFrame(None, FrameInfo(0))
        with self.assertRaises(TypeError):
            CameraFrame(object(), object())

    def test_contract_import_without_cv(self):
        code = """import sys
class BlockCV:
    def find_spec(self, fullname, *args):
        if fullname.split('.')[0] in {'cv2', 'ultralytics', 'numpy', 'torch'}:
            raise AssertionError(fullname)
sys.meta_path.insert(0, BlockCV())
from src.autonomy.vision.sources import CameraSource, CameraFrame, OpenCVSource
OpenCVSource(camera_index=1)
"""
        run = subprocess.run([sys.executable, '-B', '-c', code], capture_output=True, text=True,
                             cwd=Path(__file__).parents[1])
        self.assertEqual(run.returncode, 0, run.stderr)

    def test_demo_uses_source_boundary_with_unchanged_yolo_arguments(self):
        pixels = SimpleNamespace(shape=(480, 640, 3))
        info = FrameInfo(21, AT, ImageSize(640, 480), 'adapter-session')
        cv = Mock()
        cv.error = type('CVError', (Exception,), {})
        cv.waitKey.return_value = ord('q')
        with patch.dict(sys.modules, {'cv2': cv}), patch('src.autonomy.vision.webcam_demo.OpenCVSource') as factory, patch('src.autonomy.vision.webcam_demo.YoloDetector') as yolo:
            source = factory.return_value
            source.read.return_value = CameraFrame(pixels, info)
            yolo.return_value.detect.return_value = ()
            self.assertEqual(main(['--source', 'url', '--url', 'rtsp://example.test/live', '--model', 'local.pt', '--confidence', '0.4']), 0)
            factory.assert_called_once_with(url='rtsp://example.test/live')
            source.open.assert_called_once()
            source.close.assert_called_once()
            yolo.assert_called_once_with('local.pt', confidence=0.4, class_names=['person'], device='cpu')
            yolo.return_value.detect.assert_called_once_with(pixels, info)
        cv.VideoCapture.assert_not_called()
        cv.destroyAllWindows.assert_called_once()

    def test_demo_camera_index_cli(self):
        cv = Mock()
        cv.error = type('CVError', (Exception,), {})
        cv.waitKey.return_value = 27
        with patch.dict(sys.modules, {'cv2': cv}), patch('src.autonomy.vision.webcam_demo.OpenCVSource') as factory, patch('src.autonomy.vision.webcam_demo.YoloDetector') as yolo:
            factory.return_value.read.return_value = CameraFrame(self.pixels, FrameInfo(0))
            yolo.return_value.detect.return_value = ()
            self.assertEqual(main(['--source', 'camera', '--camera', '2']), 0)
            factory.assert_called_once_with(camera_index=2)

    def test_demo_invalid_cli_before_loading_model(self):
        for args in (['--source', 'url'], ['--source', 'bad'], ['--source', 'camera', '--url', 'http://example.test/live'],
                     ['--camera', '-1'], ['--source', 'url', '--url', 'not-a-url']):
            with patch('sys.stderr'), patch('src.autonomy.vision.webcam_demo.YoloDetector') as yolo:
                with self.assertRaises(SystemExit) as error:
                    main(args)
                self.assertEqual(error.exception.code, 2)
                yolo.assert_not_called()

    def test_demo_release_failure_still_cleans_window_and_reports_error(self):
        cv = Mock()
        cv.error = type('CVError', (Exception,), {})
        cv.waitKey.return_value = ord('q')
        with patch.dict(sys.modules, {'cv2': cv}), patch('src.autonomy.vision.webcam_demo.OpenCVSource') as factory, patch('src.autonomy.vision.webcam_demo.YoloDetector') as yolo, patch('sys.stderr') as stderr:
            factory.return_value.read.return_value = CameraFrame(self.pixels, FrameInfo(0))
            factory.return_value.close.side_effect = CameraSourceError('release failed')
            yolo.return_value.detect.return_value = ()
            self.assertEqual(main([]), 1)
            self.assertIn('Camera cleanup error', ''.join(call.args[0] for call in stderr.write.call_args_list))
        cv.destroyAllWindows.assert_called_once()

    def test_demo_keyboard_interrupt_releases_source(self):
        cv = Mock()
        cv.error = type('CVError', (Exception,), {})
        with patch.dict(sys.modules, {'cv2': cv}), patch('src.autonomy.vision.webcam_demo.OpenCVSource') as factory, patch('src.autonomy.vision.webcam_demo.YoloDetector'):
            factory.return_value.read.side_effect = KeyboardInterrupt
            self.assertEqual(main([]), 0)
            factory.return_value.close.assert_called_once()
        cv.destroyAllWindows.assert_called_once()

    def test_yolo_and_demo_do_not_construct_capture(self):
        directory = Path(__file__).parents[1] / 'src/autonomy/vision'
        for name in ('yolo.py', 'webcam_demo.py'):
            tree = ast.parse((directory / name).read_text(encoding='utf-8'))
            self.assertFalse(any(isinstance(n, ast.Attribute) and n.attr == 'VideoCapture' for n in ast.walk(tree)))


if __name__ == '__main__':
    unittest.main()
