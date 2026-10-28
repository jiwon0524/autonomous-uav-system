import unittest
from dashboard_bridge.bridge import ConnectionState


class ConnectionStateTest(unittest.TestCase):
    def setUp(self):
        self.now = 100.0
        self.state = ConnectionState(3.0, lambda: self.now)

    def test_no_message_never_connects_px4(self):
        self.assertEqual(self.state.snapshot(), {'ros2':'disconnected','px4':'disconnected'})
        self.state.spun()
        self.assertEqual(self.state.snapshot(), {'ros2':'connected','px4':'disconnected'})

    def test_receipt_timeout_and_recovery(self):
        self.state.spun()
        self.state.received()
        self.assertEqual(self.state.snapshot()['px4'], 'connected')
        self.now += 3
        self.state.spun()
        self.assertEqual(self.state.snapshot(), {'ros2':'connected','px4':'disconnected'})
        self.state.received()
        self.assertEqual(self.state.snapshot()['px4'], 'connected')

    def test_executor_stall_disconnects_both(self):
        self.state.spun()
        self.state.received()
        self.now += 3
        self.assertEqual(self.state.snapshot(), {'ros2':'disconnected','px4':'disconnected'})


class HttpEndpointTest(unittest.TestCase):
    def test_status_and_missing_route(self):
        import json
        import threading
        from http.server import ThreadingHTTPServer
        from urllib.request import urlopen
        from urllib.error import HTTPError
        from dashboard_bridge.bridge import make_handler
        state = ConnectionState()
        server = ThreadingHTTPServer(('127.0.0.1', 0), make_handler(state))
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        base = f'http://127.0.0.1:{server.server_port}'
        try:
            with urlopen(base + '/api/status', timeout=2) as response:
                self.assertEqual(json.load(response), {'ros2':'disconnected','px4':'disconnected'})
                self.assertEqual(response.headers['Cache-Control'], 'no-store')
            with self.assertRaises(HTTPError) as error:
                urlopen(base + '/not-found', timeout=2)
            self.assertEqual(error.exception.code, 404)
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=2)

if __name__ == '__main__':
    unittest.main()
