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
sidebar, d = brand((164, 314), 38, 14, 14)
for x in range(10, 164, 20): d.line((x, 78, x, 250), fill='#1d3038')
for y in range(90, 251, 20): d.line((0, y, 164, y), fill='#1d3038')
d.ellipse((18, 95, 146, 223), outline='#344b55')
for x,y in [(37,119),(127,119),(37,199),(127,199)]:
    d.line((82,159,x,y), fill='#8b99a3', width=2)
    d.ellipse((x-25,y-25,x+25,y+25), outline='#71848e')
    d.ellipse((x-3,y-3,x+3,y+3), outline='#a6b6be')
d.polygon([(72,147),(82,140),(92,147),(97,159),(92,171),(82,178),(72,171),(67,159)], outline='#d1dde2', width=2)
d.line((78,133,86,133), fill='#ff7849', width=2)
d.text((14, 260), 'FIRESCENE', font=font(15, True), fill='#E9EEF2')
d.text((14, 285), 'HARDWARE PREVIEW 02', font=font(8, True), fill='#8B99A3')
sidebar.save(out / 'sidebar.bmp')
header, d = brand((150, 57), 34, 8, 11)
d.text((49, 14), 'FireScene GCS', font=font(11, True), fill='#E9EEF2')
d.text((49, 33), 'MISSION CONTROL', font=font(8), fill='#8B99A3')
header.save(out / 'header.bmp')
