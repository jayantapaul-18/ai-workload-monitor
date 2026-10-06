#!/usr/bin/env python3
"""Regenerate all app icons (tray, window, install bundle) from the logo source."""

from __future__ import annotations

import struct
import zlib
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "public" / "app-logo-source.png"
ICONS = ROOT / "src-tauri" / "icons"
PUBLIC = ROOT / "public"


def square_crop(img: Image.Image, size: int) -> Image.Image:
    w, h = img.size
    side = min(w, h)
    left = (w - side) // 2
    top = (h - side) // 2
    cropped = img.crop((left, top, left + side, top + side))
    return cropped.resize((size, size), Image.Resampling.LANCZOS)


def write_png_sizes(base: Image.Image) -> None:
    sizes = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 512,
        "Square30x30Logo.png": 30,
        "Square44x44Logo.png": 44,
        "Square71x71Logo.png": 71,
        "Square89x89Logo.png": 89,
        "Square107x107Logo.png": 107,
        "Square142x142Logo.png": 142,
        "Square150x150Logo.png": 150,
        "Square284x284Logo.png": 284,
        "Square310x310Logo.png": 310,
        "StoreLogo.png": 50,
    }
    ICONS.mkdir(parents=True, exist_ok=True)
    for name, size in sizes.items():
        square_crop(base, size).save(ICONS / name)


def write_public_assets(base: Image.Image) -> None:
    PUBLIC.mkdir(parents=True, exist_ok=True)
    square_crop(base, 1024).save(PUBLIC / "icon-source.png")
    square_crop(base, 128).save(PUBLIC / "logo.png")
    square_crop(base, 64).save(PUBLIC / "logo-64.png")
    square_crop(base, 32).save(PUBLIC / "favicon.png")


def write_ico(base: Image.Image, path: Path) -> None:
  """Write a multi-size ICO without extra dependencies."""
  ico_sizes = [16, 24, 32, 48, 64, 128, 256]
  images = [square_crop(base, s).convert("RGBA") for s in ico_sizes]

  entries = []
  image_data = []
  offset = 6 + 16 * len(images)

  for img in images:
    w, h = img.size
    png_bytes = _png_bytes(img)
    entries.append((w, h, len(png_bytes), offset))
    image_data.append(png_bytes)
    offset += len(png_bytes)

  out = bytearray()
  out += struct.pack("<HHH", 0, 1, len(images))
  for w, h, size, off in entries:
    width = 0 if w >= 256 else w
    height = 0 if h >= 256 else h
    out += struct.pack("<BBBBHHII", width, height, 0, 0, 1, 32, size, off)
  for blob in image_data:
    out += blob

  path.write_bytes(out)


def _png_bytes(img: Image.Image) -> bytes:
  import io

  buf = io.BytesIO()
  img.save(buf, format="PNG")
  return buf.getvalue()


def write_icns(base: Image.Image, path: Path) -> None:
  """Minimal ICNS writer with common macOS icon types."""
  types = [
    (b"icp4", 16),
    (b"icp5", 32),
    (b"icp6", 64),
    (b"ic07", 128),
    (b"ic08", 256),
    (b"ic09", 512),
    (b"ic10", 1024),
  ]

  chunks = []
  for fourcc, size in types:
    png = _png_bytes(square_crop(base, size).convert("RGBA"))
    data = fourcc + struct.pack(">I", len(png)) + png
    pad = (4 - (len(data) % 4)) % 4
    data += b"\x00" * pad
    chunks.append(data)

  body = b"".join(chunks)
  header = b"icns" + struct.pack(">I", 8 + len(body))
  path.write_bytes(header + body)


def main() -> None:
  if not SOURCE.exists():
    raise SystemExit(f"Missing logo source: {SOURCE}")

  base = Image.open(SOURCE).convert("RGBA")
  write_png_sizes(base)
  write_public_assets(base)
  write_ico(base, ICONS / "icon.ico")
  write_icns(base, ICONS / "icon.icns")
  print(f"Icons regenerated in {ICONS}")


if __name__ == "__main__":
  main()
