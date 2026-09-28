#!/usr/bin/env python3
"""Generate resources/data/demo_sky.hdr — 64x32 RLE-encoded RADIANCE gradient
sky (N5 HDRI band fixture). Stdlib-only; writes the new-style per-scanline
RLE layout by hand. Values span 0.2..3.0 linear so >1.0 regions exist (the
HDR proof consumed by io-hdr tests and E507).

Run: python3 scripts/gen_hdr.py   (idempotent, deterministic)
"""

import math
import sys
from pathlib import Path

W, H = 64, 32
OUT = Path(__file__).resolve().parent.parent / "resources" / "data" / "demo_sky.hdr"


def rgbe(r, g, b):
    """f32 RGB -> RGBE quad (shared exponent; standard RADIANCE encoding).

    f = c8/256 * 2^(E-128)  =>  pick E so all channels land in [0, 1).
    """
    v = max(r, g, b)
    if v < 1e-32:
        return (0, 0, 0, 0)
    E = math.ceil(math.log2(v)) + 128
    scale = 2.0 ** (E - 128) / 256.0
    r8 = min(255, int(round(r / scale)))
    g8 = min(255, int(round(g / scale)))
    b8 = min(255, int(round(b / scale)))
    return (r8, g8, b8, E)


def rle_channel(vals):
    """Encode one channel (w bytes) with the RADIANCE run-length scheme."""
    out = bytearray()
    i = 0
    n = len(vals)
    while i < n:
        run = 1
        while i + run < n and vals[i + run] == vals[i] and run < 127:
            run += 1
        if run >= 3:  # repeat run: count byte < 128
            out.append(run)
            out.append(vals[i])
            i += run
        else:  # literals: count byte >= 128
            j = i
            lit = bytearray()
            while j < n and len(lit) < 128:
                # stop literal run if a decent repeat starts
                if j + 2 < n and vals[j] == vals[j + 1] == vals[j + 2]:
                    break
                lit.append(vals[j])
                j += 1
            out.append(128 + len(lit))
            out.extend(lit)
            i = j
    return bytes(out)


def main():
    header = (
        "#?RADIANCE\n"
        "FORMAT=32-bit_rle_rgbe\n"
        "SOFTWARE=scripts/gen_hdr.py (VisiaEngine N5 fixture)\n"
        "\n"
        "-Y %d +X %d\n" % (H, W)
    ).encode("ascii")
    body = bytearray()
    for y in range(H):
        # Vertical gradient 0.2 (top) -> 3.0 (bottom) with a soft "sun" bulge.
        rows = []
        for x in range(W):
            t = float(y) / (H - 1)
            base = 0.2 + 2.8 * t * t  # quadratic ramp 0.2 .. 3.0
            sun = 0.6 * math.exp(-(((x - 48) / 6.0) ** 2) - (((y - 8) / 4.0) ** 2))
            r = base + sun
            g = base * 0.92 + sun * 0.95
            b = base * 0.8 + sun * 0.8
            rows.append(rgbe(r, g, b))
        body += bytes([2, 2, (H >> 8) & 0xFF, H & 0xFF])  # scanline magic
        for ch in range(4):
            body += rle_channel([q[ch] for q in rows])
    OUT.write_bytes(header + bytes(body))
    size = OUT.stat().st_size
    print("wrote %s (%dx%d, %d bytes)" % (OUT, W, H, size))
    return 0


if __name__ == "__main__":
    sys.exit(main())
