#!/usr/bin/env python3
"""Local test fixture; not included in distribution. No command receiver."""
import json
import socket
import time

with socket.socket() as server:
    server.bind(('127.0.0.1', 8765))
    server.listen(1)
    print('Local fixture listening on 127.0.0.1:8765', flush=True)
    conn, _ = server.accept()
    with conn:
        while True:
            msg = {'protocol_version': 1, 'device_id': 'local-fixture', 'timestamp_ms': int(time.time() * 1000), 'connection_state': 'online', 'ai_status': 'running', 'camera_status': 'offline', 'mapping_status': 'offline'}
            try:
                conn.sendall((json.dumps(msg) + '\n').encode())
                time.sleep(1)
            except (BrokenPipeError, ConnectionResetError):
                break
