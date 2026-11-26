#!/usr/bin/env python3
"""Development-only loopback fixture. Requires Pillow; sends synthetic imagery."""
import argparse
import io
import json
import socket
import struct
import time
from PIL import Image, ImageDraw

parser = argparse.ArgumentParser()
parser.add_argument('--case', choices=['normal', 'multiple', 'mismatch', 'stale', 'stop', 'invalid', 'flood'], default='normal')
case = parser.parse_args().case

with socket.socket() as server:
    server.bind(('127.0.0.1', 8766))
    server.listen(1)
    print('DEVELOPMENT synthetic camera: 127.0.0.1:8766', flush=True)
    connection, _ = server.accept()
    with connection:
        for frame in range(1000000):
            image = Image.new('RGB', (640, 480), '#17263a')
            draw = ImageDraw.Draw(image)
            draw.text((20, 20), f'DEVELOPMENT FIXTURE {frame}', fill='white')
            draw.rectangle((64, 96, 256, 288), outline='orange', width=3)
            out = io.BytesIO()
            image.save(out, format='JPEG', quality=75)
            jpeg = out.getvalue()
            header = {'protocol_version':1, 'device_id':'fixture', 'frame_id':frame, 'timestamp_ms':int(time.time()*1000), 'width':640, 'height':480, 'model_id':'fixture', 'detections':[{'object_id':'1','class_id':'test','class_name':'TEST ONLY','confidence':0.75,'bbox':[0.1,0.2,0.3,0.4],'tracking_id':'1','depth_m':None}]}
            header['detection_frame_id'] = frame
            header['detection_timestamp_ms'] = header['timestamp_ms']
            if case == 'multiple':
                header['detections'].append({**header['detections'][0], 'object_id':'2', 'bbox':[0.5,0.4,0.2,0.2]})
            if case == 'mismatch':
                header['detection_frame_id'] = frame + 1
            if case == 'stale':
                header['timestamp_ms'] -= 10000
                header['detection_timestamp_ms'] = header['timestamp_ms']
            if case == 'invalid':
                header['detections'][0]['bbox'] = [0.9, 0.9, 0.5, 0.5]
            meta = json.dumps(header).encode('utf-8')
            try:
                connection.sendall(struct.pack('!II', len(meta), len(jpeg)) + meta + jpeg)
                if case == 'stop':
                    time.sleep(4)
                else:
                    time.sleep(0.01 if case == 'flood' else 0.2)
            except (BrokenPipeError, ConnectionResetError):
                break
