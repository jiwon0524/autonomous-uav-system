#!/usr/bin/env python3
"""Read the local key and check VWorld responses without printing credentials or URLs."""
import json
import os
from pathlib import Path
import re
import sys
import urllib.error
import urllib.parse
import urllib.request

CONFIG = Path(os.environ.get('XDG_CONFIG_HOME', Path.home() / '.config')) / 'org.firescene.gcs' / 'vworld-api-key'

def fetch(url, params, limit=2_000_000):
    request = urllib.request.Request(url + '?' + urllib.parse.urlencode(params), headers={'User-Agent': 'FireScene-GCS-map-check/1.0'})
    with urllib.request.urlopen(request, timeout=15) as response:
        data = response.read(limit + 1)
        if len(data) > limit:
            raise ValueError('response too large')
        return response.status, response.headers.get_content_type(), data

def check(name, action):
    try:
        detail = action()
        print(f'{name}: {detail}')
        return True
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError):
        # urllib exceptions can contain the request URL and key: never print them.
        print(f'{name}: FAILED (network, authentication, or unexpected response)')
        return False

def main():
    try:
        key = CONFIG.read_text(encoding='utf-8').strip()
        if not key or len(key) > 256: raise ValueError('missing key')
    except (OSError, ValueError):
        print('Local key: unavailable; configure it in FireScene settings.')
        return 1
    def tile(layer, ext, signature):
        status, mime, data = fetch('https://api.vworld.kr/req/wmts/1.0.0/' + urllib.parse.quote(key, safe='') + f'/{layer}/12/1582/3492.{ext}', {})
        if status != 200 or not data.startswith(signature): raise ValueError('invalid tile')
        return f'HTTP {status}, {mime}, {len(data)} bytes'
    def search():
        _, _, data = fetch('https://api.vworld.kr/req/search', {'service':'search','request':'search','version':'2.0','crs':'EPSG:4326','size':'3','page':'1','query':'서울시청','type':'place','format':'json','errorformat':'json','key':key})
        result = json.loads(data)['response']
        if result['status'] != 'OK' or not result.get('result', {}).get('items'): raise ValueError('no result')
        return f"OK, {len(result['result']['items'])} results"
    def building():
        _, _, data = fetch('https://api.vworld.kr/req/data', {'service':'data','request':'GetFeature','version':'2.0','data':'LT_C_SPBD','key':key,'format':'json','errorformat':'json','geometry':'true','attribute':'true','size':'5','page':'1','crs':'EPSG:4326','geomFilter':'BOX(126.977,37.565,126.979,37.567)'})
        result = json.loads(data)['response']
        features = result.get('result', {}).get('featureCollection', {}).get('features', [])
        if result['status'] != 'OK' or not features or not features[0].get('geometry'): raise ValueError('no geometry')
        return f'OK, {len(features)} building geometries'
    def sdk():
        _, mime, data = fetch('https://map.vworld.kr/js/webglMapInit.js.do', {'version':'3.0','apiKey':key})
        text = data.decode('utf-8', errors='replace')
        if not re.search(r"vworldIsValid\s*=\s*['\"]true['\"]", text) or 'WSViewerStartup.js' not in text: raise ValueError('invalid sdk response')
        return f'API 3.0 initializer returned, {mime}, {len(data)} bytes; rendering and domain authentication remain unverified'
    results = [check('Base tile', lambda: tile('Base','png',b'\x89PNG\r\n\x1a\n')), check('Satellite tile', lambda: tile('Satellite','jpeg',b'\xff\xd8\xff')), check('Place search',search), check('Building data',building), check('3D SDK',sdk)]
    return 0 if all(results) else 1

if __name__ == '__main__':
    sys.exit(main())
