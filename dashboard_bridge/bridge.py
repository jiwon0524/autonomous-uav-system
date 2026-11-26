"""Read-only ROS2 connection status bridge; no flight commands or telemetry."""
import argparse
import json
import math
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

TOPIC = '/fmu/out/vehicle_status_v4'
DEFAULT_TIMEOUT = 3.0


class ConnectionState:
    def __init__(self, timeout=DEFAULT_TIMEOUT, clock=time.monotonic):
        self.timeout = timeout
        self.clock = clock
        self.last_message = None
        self.last_spin = None
        self.lock = threading.Lock()

    def received(self):
        with self.lock:
            self.last_message = self.clock()

    def spun(self):
        with self.lock:
            self.last_spin = self.clock()

    def snapshot(self):
        with self.lock:
            now = self.clock()
            ros2 = self.last_spin is not None and now - self.last_spin < self.timeout
            px4 = ros2 and self.last_message is not None and now - self.last_message < self.timeout
            return {'ros2': 'connected' if ros2 else 'disconnected',
                    'px4': 'connected' if px4 else 'disconnected'}


def make_handler(state):
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path != '/api/status':
                self.send_error(404)
                return
            payload = json.dumps(state.snapshot()).encode('utf-8')
            self.send_response(200)
            self.send_header('Content-Type', 'application/json; charset=utf-8')
            self.send_header('Cache-Control', 'no-store')
            self.send_header('Content-Length', str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, *_args):
            pass  # Polling is not an event log.
    return Handler


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--host', default='127.0.0.1')
    parser.add_argument('--port', type=int, default=8765)
    parser.add_argument('--timeout', type=float, default=DEFAULT_TIMEOUT)
    args = parser.parse_args()
    if not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error('--timeout must be positive and finite')
    # Import only at runtime so connection logic can be tested without ROS installed.
    import rclpy
    from rclpy.node import Node
    from rclpy.qos import qos_profile_sensor_data
    from px4_msgs.msg import VehicleStatus

    rclpy.init()
    node = Node('dashboard_status_bridge', enable_rosout=False, start_parameter_services=False)
    state = ConnectionState(args.timeout)
    node.create_subscription(VehicleStatus, TOPIC, lambda _message: state.received(), qos_profile_sensor_data)
    server = ThreadingHTTPServer((args.host, args.port), make_handler(state))
    server.daemon_threads = True
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    print(f'Dashboard status: http://{args.host}:{args.port}/api/status', flush=True)
    print(f'Read-only subscription: {TOPIC}; timeout={args.timeout}s', flush=True)
    try:
        while rclpy.ok():
            rclpy.spin_once(node, timeout_sec=0.1)
            state.spun()
    except KeyboardInterrupt:
        pass
    finally:
        server.shutdown()
        server.server_close()
        node.destroy_node()
        if rclpy.ok():
            rclpy.shutdown()


if __name__ == '__main__':
    main()
