#!/usr/bin/env python3
"""Shared base for the agent single-source generators (skills / rules / mcp / memory).

Extracted from scripts/skills-links.py so the per-layer generators stay small and
do not each re-implement the guard / classify / link / backup logic (adjudication 2-B:
"per-layer small generators + shared base", avoid over-generalising into one mega-script).

Exit codes (consistent across all gate-* scripts so CI can branch on them):
  0 = clean, 1 = findings (check) / actions applied (sync), 2 = guard violation (nothing written).
Stdlib only, Python 3.8 compatible (CI host system python3 is 3.8).
"""

import datetime
import json
import os
import re
import shutil
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

EXIT_OK = 0
EXIT_FINDINGS = 1
EXIT_GUARD = 2

BACKUP_KEEP = 5

# link classification states
OK = "ok"
MISSING = "missing"
DANGLING = "dangling"
EXISTS = "exists"
WRONG_ABS = "wrong-target/absolute"
WRONG_FOREIGN = "wrong-target/foreign"
RED = (MISSING, DANGLING, EXISTS, WRONG_ABS, WRONG_FOREIGN)


def load_json(path):
    with open(path, "r") as handle:
        return json.load(handle)


def guard_reasons(target_dir, canonical_real):
    """Why writing into target_dir would be unsafe (empty list == safe).

    Mirrors the skills-layer guard: never write into a symlink, never write into
    the canonical tree (would corrupt the single source), never write under a
    symlinked parent.
    """
    reasons = []
    if os.path.islink(target_dir):
        reasons.append("target dir is a symlink (%s)" % os.readlink(target_dir))
        return reasons
    target_real = os.path.realpath(target_dir)
    if target_real == canonical_real or target_real.startswith(canonical_real + os.sep):
        reasons.append("target dir resolves inside the canonical tree (%s)" % target_real)
    probe = os.path.dirname(os.path.abspath(target_dir))
    while probe and probe != ROOT and probe != os.path.dirname(probe):
        if os.path.islink(probe):
            reasons.append("parent is a symlink (%s)" % probe)
            break
        probe = os.path.dirname(probe)
    if os.path.islink(os.path.dirname(canonical_real)):
        reasons.append("canonical parent is a symlink (%s)" % os.path.dirname(canonical_real))
    return reasons


def classify_link(link, canonical_real):
    """State of one expected symlink: ok | missing | dangling | exists | wrong-target/*."""
    if not os.path.lexists(link):
        return MISSING
    if not os.path.islink(link):
        return EXISTS
    if not os.path.exists(link):
        return DANGLING
    resolved = os.path.realpath(link)
    if resolved == canonical_real or resolved.startswith(canonical_real + os.sep):
        return OK
    return WRONG_FOREIGN


def make_link(src, link):
    """Create a relative symlink link -> src. Raises RuntimeError with a --copy hint on Windows."""
    rel = os.path.relpath(src, os.path.dirname(link))
    try:
        os.symlink(rel, link, target_is_directory=os.path.isdir(src))
    except (OSError, NotImplementedError) as exc:
        winerror = getattr(exc, "winerror", None)
        hint = "--copy" if winerror == 1314 else "administrator/developer-mode privileges"
        raise RuntimeError("cannot create symlink %s (%s); rerun with %s" % (link, exc, hint))


def backup_name(target_dir, name):
    stamp = datetime.datetime.now().strftime("%Y%m%dT%H%M%S%f")
    return os.path.join(target_dir, "%s.bak-%s" % (name, stamp))


def prune_backups(target_dir, name):
    prefix = name + ".bak-"
    if not os.path.isdir(target_dir):
        return
    olds = sorted(entry for entry in os.listdir(target_dir) if entry.startswith(prefix))
    for entry in olds[:-BACKUP_KEEP]:
        path = os.path.join(target_dir, entry)
        if os.path.isdir(path) and not os.path.islink(path):
            shutil.rmtree(path, ignore_errors=True)


def copy_tree(src, dst):
    if os.path.islink(dst):
        os.unlink(dst)
    elif os.path.isdir(dst):
        shutil.rmtree(dst)
    shutil.copytree(src, dst)


def ensure_dir(path):
    os.makedirs(path, exist_ok=True)


def iter_rel_files(canonical_root, suffix):
    """Yield relative posix paths of every file under canonical_root ending in suffix."""
    out = []
    for dirpath, _dirnames, filenames in os.walk(canonical_root):
        for filename in filenames:
            if not filename.endswith(suffix):
                continue
            full = os.path.join(dirpath, filename)
            out.append(os.path.relpath(full, canonical_root).replace(os.sep, "/"))
    return sorted(out)


def marker_line_count(text, marker):
    """Count marker lines anchored at both ends (prose mentions do not count)."""
    return len(re.findall(r"(?m)^" + re.escape(marker) + r"[ \t]*$", text))


def single_marker_ok(prefix, text, begin, end):
    """True when the region is absent (0 markers) or exactly one line each.

    Duplicated or half-present markers mean an earlier rewrite leaked markers;
    writing again would compound the damage, so callers must return EXIT_GUARD.
    """
    nb = marker_line_count(text, begin)
    ne = marker_line_count(text, end)
    if (nb, ne) in ((0, 0), (1, 1)):
        return True
    print("%s ✗ marker-guard: BEGIN=%d END=%d (each must be one line, or both absent)" % (prefix, nb, ne))
    return False


HUB_LIMIT = 30000
HUB_WARN = 28500  # 95% of the limit: the hub must keep headroom for new entries


def hub_budget(path):
    """D47: the hub must fit the host's per-turn injection ceiling.

    Both units are checked because the observed cut was measured in bytes while
    editors count characters; CJK text makes the two diverge.
    """
    text = open(path, "r", encoding="utf-8").read()
    chars, nbytes = len(text), len(text.encode("utf-8"))
    if chars > HUB_LIMIT or nbytes > HUB_LIMIT:
        print("GATE-HUB ✗ %s %d chars / %d bytes > %d (see D47)"
              % (os.path.basename(path), chars, nbytes, HUB_LIMIT))
        return False
    if chars > HUB_WARN or nbytes > HUB_WARN:
        print("GATE-HUB ! %s %d chars / %d bytes (warn %d, limit %d)"
              % (os.path.basename(path), chars, nbytes, HUB_WARN, HUB_LIMIT))
    return True


def gate(prefix, ok, msg):
    mark = "✓" if ok else "✗"
    print("%s %s %s" % (prefix, mark, msg))


def die_guard(prefix, target_dir, reasons):
    print("%s ✗ guard %s: %s" % (prefix, target_dir, "; ".join(reasons)))
    sys.exit(EXIT_GUARD)
