#!/usr/bin/env python3
"""Generate lightweight NSIS BMP artwork from the existing FireScene icon.
Requires Pillow only when regenerating assets; never bundled with the application.
"""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
root = Path(__file__).resolve().parents[1]
out = root / 'src-tauri/installer'
out.mkdir(parents=True, exist_ok=True)
icon = Image.open(root / 'src-tauri/icons/128x128.png').convert('RGBA')
def font(size, bold=False):
    name = '/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf' if bold else '/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf'
    return ImageFont.truetype(name, size)
def brand(size, mark_size, x, y):
    image = Image.new('RGB', size, '#0B0F12')
    image.paste(icon.resize((mark_size, mark_size), Image.Resampling.LANCZOS), (x, y), icon.resize((mark_size, mark_size), Image.Resampling.LANCZOS))
    return image, ImageDraw.Draw(image)
sidebar, d = brand((164, 314), 54, 18, 25)
d.line((18, 103, 146, 103), fill='#61B7FF', width=2)
d.text((18, 119), 'FireScene', font=font(20, True), fill='#E9EEF2')
d.text((18, 147), 'GCS', font=font(19, True), fill='#E9EEF2')
d.text((18, 185), 'AEROSPACE', font=font(10, True), fill='#8B99A3')
d.text((18, 201), 'MISSION CONTROL', font=font(9, True), fill='#8B99A3')
d.text((18, 266), 'HARDWARE', font=font(10, True), fill='#61B7FF')
d.text((18, 283), 'PREVIEW  /  0.1.0', font=font(9), fill='#8B99A3')
sidebar.save(out / 'sidebar.bmp')
header, d = brand((150, 57), 34, 8, 11)
d.text((49, 14), 'FireScene GCS', font=font(11, True), fill='#E9EEF2')
d.text((49, 33), 'MISSION CONTROL', font=font(8), fill='#8B99A3')
header.save(out / 'header.bmp')
