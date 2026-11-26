#!/usr/bin/env python3
"""Illustrate implemented NSIS page structure, not Windows renderings."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

root = Path(__file__).resolve().parents[1]
out = root / 'docs/phase6-concepts/installer-pages'
out.mkdir(parents=True, exist_ok=True)
font = '/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf'
bold = '/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf'
f = lambda size, heavy=False: ImageFont.truetype(bold if heavy else font, size)
sidebar = Image.open(root / 'src-tauri/installer/sidebar.bmp').convert('RGB').resize((164, 314))
header = Image.open(root / 'src-tauri/installer/header.bmp').convert('RGB')
for key, title, subtitle in [
    ('welcome', 'FIRESCENE', 'AEROSPACE MISSION CONTROL'),
    ('configuration', 'FIRESCENE / Installation location', 'Choose where FireScene GCS will be installed.'),
    ('deployment', 'FIRESCENE / Deployment', 'Installing FireScene GCS and required components.'),
    ('completion', 'Installation complete', 'FireScene GCS 0.1.0 is installed.'),
    ('uninstaller', 'FIRESCENE / Uninstall', 'Review removal and optional user data deletion.'),
]:
    im = Image.new('RGB', (650, 480), '#eef0f2')
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, 649, 29), fill='#20282d')
    d.text((12, 7), 'FireScene GCS  —  IMPLEMENTATION-BASED MOCKUP', font=f(11), fill='#e9eef2')
    if key in ('welcome', 'completion'):
        d.rectangle((0, 30, 649, 421), fill='#0b0f12')
        im.paste(sidebar.resize((205, 392)), (0, 30))
        d.text((225, 69), title, font=f(24, True), fill='#e9eef2')
        d.text((225, 115), subtitle, font=f(12), fill='#8b99a3')
        if key == 'welcome':
            d.text((225, 166), 'HARDWARE PREVIEW 02 / READ ONLY', font=f(11), fill='#61b7ff')
            d.text((225, 210), 'Install the read-only ground control station.', font=f(12), fill='#e9eef2')
        else:
            d.text((225, 162), 'Hardware Preview 02 is read only.', font=f(12), fill='#e9eef2')
            d.text((225, 183), 'Flight commands are locked.', font=f(12), fill='#e9eef2')
            d.text((225, 245), '☐  Run FireScene GCS', font=f(12), fill='#e9eef2')
            d.text((225, 274), '☐  Create desktop shortcut', font=f(12), fill='#e9eef2')
    else:
        d.rectangle((0, 30, 649, 104), fill='#0b0f12')
        im.paste(header, (500, 38))
        d.text((18, 43), title, font=f(15, True), fill='#e9eef2')
        d.text((18, 74), subtitle, font=f(11), fill='#b5c0c5')
        if key == 'configuration':
            d.text((30, 150), 'Destination folder', font=f(13, True), fill='#243038')
            d.rectangle((30, 180, 490, 214), fill='white', outline='#7d898f')
            d.text((40, 190), r'C:\Users\...\AppData\Local\FireScene GCS', font=f(11), fill='#253038')
            d.rectangle((505, 180, 615, 214), fill='#e6eaec', outline='#8b999f')
            d.text((526, 190), 'Browse...', font=f(11), fill='#253038')
            d.text((30, 260), 'Existing version and start menu use separate native pages.', font=f(11), fill='#53616a')
        elif key == 'deployment':
            d.text((30, 145), 'Installing...', font=f(13, True), fill='#243038')
            d.rectangle((30, 180, 615, 204), fill='white', outline='#7d898f')
            d.rectangle((32, 182, 325, 202), fill='#61b7ff')
            d.text((30, 235), 'Real NSIS progress and installation details appear here.', font=f(11), fill='#53616a')
        else:
            d.text((30, 150), 'Remove FireScene GCS from this computer?', font=f(13), fill='#243038')
            d.text((30, 200), '☐  Delete app data (optional)', font=f(12), fill='#243038')
    d.rectangle((0, 422, 649, 479), fill='#e6eaec')
    d.text((14, 443), 'FIRESCENE  |  AEROSPACE MISSION CONTROL', font=f(9), fill='#59676d')
    for x, text in [(390, 'Back'), (475, 'Next'), (560, 'Cancel')]:
        d.rectangle((x, 436, x+75, 466), fill='#f7f8f9', outline='#8b999f')
        d.text((x+16, 444), text, font=f(10), fill='#253038')
    im.save(out / f'{key}.png')
    print(out / f'{key}.png')
