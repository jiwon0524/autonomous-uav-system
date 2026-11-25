#!/usr/bin/env python3
"""Development-only synthetic point cloud server; loopback only."""
import argparse
import json
import math
import socket
import struct
import time
p=argparse.ArgumentParser()
p.add_argument('--case', choices=['normal','session','invalid','length','flood','stop'], default='normal')
case=p.parse_args().case
with socket.socket() as server:
    server.bind(('127.0.0.1',8767));server.listen(1)
    print('DEVELOPMENT synthetic point cloud: 127.0.0.1:8767',flush=True)
    conn,_=server.accept()
    with conn:
        for frame in range(100000):
            payload=bytearray()
            for i in range(4000):
                a=i*0.0314
                payload.extend(struct.pack('<fffBBB',math.cos(a)*4, math.sin(a)*4, (i%200)*.025, i%256, (i//8)%256, 210))
            if case=='invalid':payload[:4]=struct.pack('<f',float('nan'))
            h={'protocol_version':1,'device_id':'fixture','session_id':'session-2' if case=='session' and frame>1 else 'session-1','map_id':'fixture-map','frame_id':frame,'timestamp_ms':int(time.time()*1000),'coordinate_frame':'local_map','point_count':4000,'position_encoding':'xyz_f32_le','color_encoding':'rgb8','payload_length':len(payload),'map_revision':frame,'data_type':'snapshot','mission_id':None}
            meta=json.dumps(h).encode()
            try:
                conn.sendall(struct.pack('!II',len(meta),len(payload) + (1 if case=='length' else 0))+meta+payload)
                time.sleep(6 if case=='stop' else .01 if case=='flood' else 1)
            except (BrokenPipeError,ConnectionResetError):break
