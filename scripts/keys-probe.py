#!/usr/bin/env python3
"""Band K (2026-10-08): machine-driven input probe for the interactive examples.

XTEST-injects keys / wheel / drags into a real window on a PRIVATE Xvfb display and
fails when the window stays alive but the picture never changes -- the class of defect
that pixel gates cannot see ("the handler is not wired", "the key changes a variable
nobody reads"). It is the T3-input half turned into T2; what remains human is listed
in .agents/rules/common/testing.md.

usage:
  python3 scripts/keys-probe.py --all
  python3 scripts/keys-probe.py E506_postprocessing key 4
  python3 scripts/keys-probe.py E510_route_flow drag 140 40 wheel 1 3

Rules baked in from the session that validated this harness:
  * private display, never $DISPLAY -- the tool channel must not shadow the user
    channel (PIT-22), and a probe that renders onto someone's desktop is a lie.
  * every assertion happens BEFORE cleanup; checking a deleted tree or a killed
    window is vacuous-true.
  * per-case settle/gap: E511's clock moved 0.21% over 1.8s and ~1% over 4s, so the
    animation case gets a 4s gap and a lower floor. Reading a slow clock at 1.8s is
    how you call working code broken.
  * SKIP (exit 0) is for a missing toolchain only -- and only by default. `--strict`
    turns a skip into exit 1: xwd comes from the SYSTEM package x11-apps (measured
    2026-10-08, not from pixi), so a runner that installed only `xvfb` would otherwise
    skip this entire probe and still report green (PIT-25 family). ci.yml passes
    --strict for exactly that reason.
  * a live window with no pixel change is exit 1 with the reason.
"""
import os
import re
import shutil
import struct
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ENV = os.path.join(ROOT, ".pixi", "envs", "default")
EXDIR = os.path.join(ROOT, "target", "debug", "examples")
WORK = os.path.join(ROOT, "target", "keys-probe")
XWD = "xwd"  # re-bound in main() to the discovered path (PATH lookup, not /usr/bin)
INJECTOR = os.path.join(WORK, "xinject")
SETTLE = 5.0  # first frame + window map

# (stem, actions, gap, min_changed_pct) -- actions are xinject argv tuples.
# "noop" = focus only, so a delta between two captures can only be the app's clock.
# The four window rows below (E303/E306/E504/E901) must NEVER be "noop": their
# idle frame is static (input-gated redraw or deterministic re-render), so noop
# would print 0.00% and gate on the harness, not on the app. Each drives
# drag + wheel + one meaningful key.
# Deliberately EXCLUDED (the Escape-must-close assertion would be permanently red):
#   E201_load_gltf   -- Escape is a zoom key (examples/rs/E201_load_gltf.rs:52
#                      `Escape => dist *= 0.9`); the window never closes on it.
#   E501_shadow_demo -- no keyboard handling at all (only MouseWheel); same
#                      never-closes reason.
# Floors (2026-10-09, 5 consecutive --all runs; evidence
# .omo/evidence/keys-probe-8row-2026-10-09/): floor = 0.4 x the row's weakest
# measured action min (60% relative margin, floored): E303 15.46 -> 6.0,
# E306 25.07 -> 10.0, E504 13.24 -> 5.0, E901 5.96 -> 2.3. run_case checks
# every action in a row against the floor individually, so this is a
# per-action contract, not a per-row average.
CASES = [
    ("E506_postprocessing", [("key", "4")], 2.0, 0.5),
    ("E507_hdr_tonemap", [("key", "2"), ("key", "3"), ("key", "1")], 2.0, 0.5),
    ("E510_route_flow", [("drag", "140", "40"), ("wheel", "1", "3")], 2.0, 0.5),
    ("E511_gate", [("noop", "0")], 4.0, 0.4),
    ("E303_split_screen", [("drag", "140", "40"), ("wheel", "1", "3"), ("key", "2")], 2.0, 6.0),
    ("E306_map_controls", [("drag", "140", "40"), ("wheel", "1", "3"), ("key", "r")], 2.0, 10.0),
    ("E504_glass_water", [("drag", "140", "40"), ("wheel", "1", "3"), ("key", "2")], 2.0, 5.0),
    ("E901_twin_city", [("drag", "140", "40"), ("wheel", "1", "3"), ("key", "c")], 2.0, 2.3),
]


def find_tool(*names):
    """Absolute paths (the pixi env) first, then PATH."""
    for n in names:
        if os.path.isabs(n):
            if os.path.exists(n):
                return n
        else:
            hit = shutil.which(n)
            if hit:
                return hit
    return None


def skip(reason):
    print(f"KEYS-PROBE SKIP ({reason})")
    return 0


def free_display(taken):
    for n in range(79, 96):
        if f":{n}" not in taken and not os.path.exists(f"/tmp/.X11-unix/X{n}"):
            return f":{n}"
    return None


def build_injector(cc, xwd_missing_ok=True):
    """Compile scripts/xinject.c once; newer source rebuilds it."""
    src = os.path.join(ROOT, "scripts", "xinject.c")
    if os.path.exists(INJECTOR) and os.path.getmtime(INJECTOR) > os.path.getmtime(src):
        return True
    os.makedirs(WORK, exist_ok=True)
    cmd = [cc, "-O1", "-o", INJECTOR, src]
    if os.path.isdir(os.path.join(ENV, "include")):
        cmd += [f"-I{ENV}/include", f"-L{ENV}/lib", f"-Wl,-rpath,{ENV}/lib"]
    cmd += ["-lX11", "-lXtst"]
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode != 0:
        print(f"KEYS-PROBE ✗ injector build failed: {r.stderr.strip()[:400]}")
        return False
    return True


def xwd(disp, path):
    subprocess.run(
        [XWD, "-root", "-display", disp, "-out", path],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=True,
    )


def sample_points(path, step=6):
    """Return (pixel bytes, bytes-per-line, sampled (y,x) grid) from an XWD dump.

    XWD is big-endian on disk; the header's own byte order only tells where the
    *colour map* sits, so a swapped unpack would mis-size the sample grid -- guard
    it instead of silently trusting it (the diff percentage is a gate)."""
    data = open(path, "rb").read()
    hdr = struct.unpack(">25I", data[: struct.calcsize(">25I")])
    width, height, bpl = hdr[4], hdr[5], hdr[12]
    # hdr[0] is header_size INCLUDING the variable-length name (measured 107 for a
    # 6-char name), so no constant can guard it. The length identity can: a mismatch
    # means our field layout drifted, and a silently mis-sized grid is a false gate.
    off = hdr[0] + hdr[19] * 12
    if off + height * bpl != len(data):
        raise RuntimeError(f"XWD layout mismatch: off={off} + {height}*{bpl} != {len(data)}")
    px = data[off:]
    pts = [(y, x) for y in range(0, height, step) for x in range(0, width, step)]
    return px, bpl, pts


def diff(pa, pb):
    A, bpl, pts = sample_points(pa)
    B, _, _ = sample_points(pb)
    d = sum(1 for y, x in pts if A[y * bpl + x * 4 : y * bpl + x * 4 + 3] != B[y * bpl + x * 4 : y * bpl + x * 4 + 3])
    return d, len(pts)


def inject(disp, needle, *args):
    r = subprocess.run([INJECTOR, disp, needle, *args], capture_output=True, text=True)
    return r.returncode, (r.stdout + r.stderr).strip().splitlines()


def title_of(disp, needle):
    rc, out = inject(disp, needle, "name")
    for line in out:
        if line.startswith("NAME="):
            return line[5:]
    return ""


def run_case(disp, stem, actions, gap, minpct):
    # The window title is "E506 post off ..." (a space, not the file stem), so the
    # X-side needle is the E number and the binary is the full stem -- measured 2026-10-08.
    needle = stem.split("_", 1)[0]
    exe = os.path.join(EXDIR, stem)
    if not os.access(exe, os.X_OK):
        print(f"KEYS-PROBE ✗ {stem}: binary absent ({exe}); run `cargo build --examples`")
        return False
    a = os.path.join(WORK, "before.xwd")
    b = os.path.join(WORK, "after.xwd")
    env = dict(os.environ, DISPLAY=disp)
    p = subprocess.Popen([exe], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(SETTLE)
    ok = True
    try:
        if p.poll() is not None:
            print(f"KEYS-PROBE ✗ {stem}: zero-arg run is not resident (rc={p.returncode}) -- C15 dual-mode breach")
            return False
        xwd(disp, a)
        for act in actions:
            rc, msg = inject(disp, needle, *act)
            if rc != 0:
                print(f"KEYS-PROBE ✗ {stem}: injection failed {act}: {msg[-1] if msg else ''}")
                return False
            time.sleep(gap)
            xwd(disp, b)
            d, tot = diff(a, b)
            pct = 100.0 * d / tot
            print(f'KEYS-PROBE {stem} {" ".join(act)}: changed={pct:.2f}% title="{title_of(disp, needle)}"')
            if pct < minpct:
                print(f"KEYS-PROBE ✗ {stem}: window alive, nothing changed after {' '.join(act)}")
                ok = False
            os.replace(b, a)
        # Esc must close the app through the same real X path the keys used
        inject(disp, needle, "key", "Escape")
        try:
            p.wait(timeout=6)
            print(f"KEYS-PROBE {stem} Escape: exited rc={p.returncode}")
        except subprocess.TimeoutExpired:
            print(f"KEYS-PROBE ✗ {stem}: Escape did not close the window")
            ok = False
    finally:
        if p.poll() is None:
            p.kill()
    return ok


def main():
    argv = sys.argv[1:]
    strict = "--strict" in argv
    args = [a for a in argv if a != "--strict"]

    xvfb = find_tool(os.path.join(ENV, "bin", "Xvfb"), "Xvfb")
    cc = find_tool("cc", os.path.join(ENV, "bin", "x86_64-conda-linux-gnu-cc"))
    xwd = find_tool("xwd", "/usr/bin/xwd")
    xtest_h = os.path.join(ENV, "include", "X11", "extensions", "XTest.h")
    missing = [n for n, ok in (("Xvfb", bool(xvfb)), ("cc", bool(cc)), ("xwd", bool(xwd)),
                               ("XTest.h", os.path.exists(xtest_h))) if not ok]
    if missing:
        if strict:
            print(f"KEYS-PROBE ✗ --strict but missing toolchain: {', '.join(missing)}")
            return 1
        return skip(f"missing {', '.join(missing)}")
    globals()["XWD"] = xwd
    if not build_injector(cc):
        return 1
    if not os.path.isdir(EXDIR):
        print("KEYS-PROBE ✗ no example binaries; run `cargo build --examples`")
        return 1

    if not args:
        print(__doc__)
        return 2
    taken = [x for x in re.findall(r":(\d+)", os.environ.get("DISPLAY", ""))]
    disp = free_display(taken)
    if not disp:
        print("KEYS-PROBE ✗ no free display in :79..:95")
        return 1

    xvfblog = os.path.join(WORK, "xvfb.log")
    os.makedirs(WORK, exist_ok=True)
    with open(xvfblog, "wb") as lf:
        proc = subprocess.Popen([xvfb, disp, "-screen", "0", "1280x800x24"], stdout=lf, stderr=subprocess.STDOUT)
    time.sleep(1.5)
    results = {}
    try:
        if args[0] == "--all":
            for stem, acts, gap, minpct in CASES:
                results[stem] = run_case(disp, stem, acts, gap, minpct)
        else:
            # single-shot form: every remaining token belongs to one xinject action
            stem, acts = args[0], [tuple(args[1:])]
            results[stem] = run_case(disp, stem, [tuple(args[1:])], 2.0, 0.5)
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
    bad = [k for k, v in results.items() if not v]
    print(f"KEYS-PROBE {'✓' if not bad else '✗'} {len(results) - len(bad)}/{len(results)} cases"
          + (f" (failed: {', '.join(bad)})" if bad else ""))
    return 0 if not bad else 1


if __name__ == "__main__":
    sys.exit(main())
