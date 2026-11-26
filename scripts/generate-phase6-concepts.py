#!/usr/bin/env python3
"""Generate comparison mockups; these are not native installer screenshots."""
from base64 import b64encode
from pathlib import Path

root = Path(__file__).resolve().parents[1]
out = root / 'docs' / 'phase6-concepts'
out.mkdir(exist_ok=True)
airframe = b64encode((root / 'frontend/public/firescene-airframe.svg').read_bytes()).decode()
mark = b64encode((root / 'frontend/public/firescene-mark.png').read_bytes()).decode()
for key, label, accent, x, opacity in [
    ('a', 'AEROSPACE PRECISION', '#61b7ff', 430, '.8'),
    ('b', 'URBAN FIRE RESPONSE', '#ff7849', 345, '.85'),
    ('c', 'ORBITAL MISSION CONTROL', '#8b99a3', 480, '.45'),
]:
    art = f'<image href="data:image/svg+xml;base64,{airframe}" x="{x}" y="62" width="480" height="300" opacity="{opacity}"/>'
    city = '<path d="M30 330h55v-60h35v-45h34v105h40v-75h35v-35h30v110h50" fill="none" stroke="#ff7849" opacity=".65"/>' if key == 'b' else ''
    orbit = '<circle cx="665" cy="200" r="150" fill="none" stroke="#61b7ff" opacity=".4" stroke-dasharray="4 8"/><image href="data:image/png;base64,'+mark+'" x="635" y="170" width="60" height="60"/>' if key == 'c' else ''
    svg = f'''<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="820" viewBox="0 0 1000 820"><rect width="1000" height="820" fill="#080c0f"/><g font-family="sans-serif" fill="#e9eef2"><text x="35" y="35" font-size="15" letter-spacing="3">CONCEPT {key.upper()} / {label}</text><g transform="translate(30 55)"><rect width="940" height="340" fill="#0b0f12" stroke="#34444d"/>{art}{city}{orbit}<text x="30" y="45" font-size="11" letter-spacing="2" fill="{accent}">INSTALLER / CONCEPT MOCKUP</text><text x="30" y="120" font-size="36" letter-spacing="5">FIRESCENE</text><text x="30" y="150" font-size="11" letter-spacing="2">AEROSPACE MISSION CONTROL</text><text x="30" y="190" font-size="12" fill="#8b99a3">HARDWARE PREVIEW 02 · READ ONLY</text><rect x="30" y="245" width="150" height="40" fill="{accent}"/><text x="45" y="270" font-size="13" fill="#0b0f12">START SETUP →</text></g><g transform="translate(30 425)"><rect width="940" height="340" fill="#0b0f12" stroke="#34444d"/>{art}{city}{orbit}<text x="30" y="45" font-size="11" letter-spacing="2" fill="{accent}">APP LAUNCH / CONCEPT MOCKUP</text><text x="30" y="145" font-size="36" letter-spacing="5">FIRESCENE</text><text x="30" y="175" font-size="11" letter-spacing="2">AEROSPACE MISSION CONTROL</text><path d="M30 228h25" stroke="{accent}" stroke-width="2"/><text x="65" y="232" font-size="12" fill="#8b99a3">Loading interface</text></g></g></svg>'''
    (out / f'concept-{key}.svg').write_text(svg)
