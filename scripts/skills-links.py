#!/usr/bin/env python3
"""Check and repair the per-host skill activation layer.

Single source: <repo>/.agents/skills/<name>/SKILL.md -- the only editable copy.
Targets: declared in scripts/skills-targets.json. A target directory may only
contain relative symlinks back to the canonical tree; in copy mode it may
contain generated copies whose hashes are recorded in a link map.

usage:
    python3 scripts/skills-links.py check    [--copy]
    python3 scripts/skills-links.py sync     [--copy]
    python3 scripts/skills-links.py dry-run
    python3 scripts/skills-links.py list

Exit codes: 0 clean, 1 findings, 2 guard violation (nothing was written).
Stdlib only, Python 3.8 compatible (the system python3 on CI hosts is 3.8).

Rules enforced (see .omo/plans/2026-10-28-agent-skills-single-source.md):
  - every canonical skill exists in every enabled target, as a relative link
    whose realpath is exactly canonical/<name>;
  - a real directory where a link belongs is backed up, never overwritten;
  - links into the canonical tree whose skill disappeared are pruned;
  - a target directory that is itself a symlink (or lives inside the canonical
    tree) is a guard violation: writing there would corrupt the single source.
"""

import datetime
import hashlib
import json
import os
import shutil
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CONFIG_PATH = os.path.join(ROOT, "scripts", "skills-targets.json")
COPY_MAP_PATH = os.path.join(ROOT, ".codebuddy", "skills-link-map.json")
BACKUP_KEEP = 5

OK = "ok"
MISSING = "missing"
DANGLING = "dangling"
EXISTS = "exists"
WRONG_ABS = "wrong-target/absolute"
WRONG_CROSS = "wrong-target/cross-name"
WRONG_FOREIGN = "wrong-target/foreign"
EXTRA = "extra-link"
EXTRA_DIR = "extra-dir"
RED = (MISSING, DANGLING, EXISTS, WRONG_ABS, WRONG_CROSS, WRONG_FOREIGN, EXTRA, EXTRA_DIR)


def load_config():
    with open(CONFIG_PATH, "r") as handle:
        data = json.load(handle)
    skills_root = os.path.join(ROOT, data["skills_root"])
    return skills_root, data["targets"], data.get("forbidden_dirs") or []


def check_forbidden(forbidden):
    """Host directories that must not hold skills.

    A host that reads its own skills folder BEFORE `.agents/skills/` silently
    shadows the single source for every same-named skill (Trae: `.trae/skills`
    wins over `.agents/skills`). Reported as a finding, never as a guard: we
    never write into that folder, so escalating would let one local skill file
    abort every host's sync (D50 R3).
    """
    findings = []
    for rel in forbidden:
        root = os.path.join(ROOT, rel)
        if not os.path.isdir(root):
            continue
        for dirpath, _dirnames, filenames in os.walk(root):
            if "SKILL.md" in filenames:
                findings.append((os.path.relpath(dirpath, ROOT).replace(os.sep, "/"), "shadows-single-source"))
    return findings


def discover_skills(skills_root):
    """Return {name: abs_path} for every directory holding a SKILL.md."""
    found = {}
    for dirpath, dirnames, filenames in os.walk(skills_root):
        if "SKILL.md" not in filenames:
            continue
        name = os.path.basename(dirpath)
        if name in found:
            print("GATE-SKILLS ✗ duplicate-skill %s: %s and %s" % (name, found[name], dirpath))
            sys.exit(1)
        found[name] = dirpath
    if not found:
        print("GATE-SKILLS ✗ canonical-empty %s" % skills_root)
        sys.exit(1)
    return found


def guard_reasons(skills_dir, canonical_real):
    """Why writing into skills_dir would be unsafe (empty list == safe)."""
    reasons = []
    if os.path.islink(skills_dir):
        reasons.append("target dir is a symlink (%s)" % os.readlink(skills_dir))
        return reasons
    target_real = os.path.realpath(skills_dir)
    if target_real == canonical_real or target_real.startswith(canonical_real + os.sep):
        reasons.append("target dir resolves inside the canonical tree (%s)" % target_real)
    probe = os.path.dirname(os.path.abspath(skills_dir))
    while probe and probe != ROOT and probe != os.path.dirname(probe):
        if os.path.islink(probe):
            reasons.append("parent is a symlink (%s)" % probe)
            break
        probe = os.path.dirname(probe)
    if os.path.islink(os.path.dirname(canonical_real)):
        reasons.append("canonical parent is a symlink (%s)" % os.path.dirname(canonical_real))
    return reasons


def classify(link, skill_dir, canonical_real):
    """State of one expected link: ok | missing | dangling | exists | wrong-target/*."""
    if not os.path.lexists(link):
        return MISSING
    if not os.path.islink(link):
        return EXISTS
    raw = os.readlink(link)
    if os.path.isabs(raw):
        return WRONG_ABS
    if not os.path.exists(link):
        return DANGLING
    resolved = os.path.realpath(link)
    if resolved == os.path.realpath(skill_dir):
        return OK
    if resolved.startswith(canonical_real + os.sep):
        return WRONG_CROSS
    return WRONG_FOREIGN


def extra_links(skills_dir, names, canonical_real):
    """Stale entries inside skills_dir: (name, kind) with kind "link" or "dir".

    A link counts as stale when it resolves into the canonical tree but its
    skill no longer exists; a real directory counts as stale when it looks
    like a skill copy. Backups (`<name>.bak-*`) are never stale.
    """
    stale = []
    if not os.path.isdir(skills_dir):
        return stale
    for entry in sorted(os.listdir(skills_dir)):
        if entry in names or ".bak-" in entry:
            continue
        path = os.path.join(skills_dir, entry)
        if os.path.islink(path):
            resolved = os.path.realpath(path)
            if resolved.startswith(canonical_real + os.sep):
                stale.append((entry, "link"))
        elif os.path.isdir(path) and os.path.exists(os.path.join(path, "SKILL.md")):
            stale.append((entry, "dir"))
    return stale


def backup_name(skills_dir, name):
    stamp = datetime.datetime.now().strftime("%Y%m%dT%H%M%S%f")
    return os.path.join(skills_dir, "%s.bak-%s" % (name, stamp))


def prune_backups(skills_dir, name):
    prefix = name + ".bak-"
    if not os.path.isdir(skills_dir):
        return
    olds = sorted(entry for entry in os.listdir(skills_dir) if entry.startswith(prefix))
    for entry in olds[:-BACKUP_KEEP]:
        path = os.path.join(skills_dir, entry)
        if os.path.isdir(path) and not os.path.islink(path):
            shutil.rmtree(path, ignore_errors=True)


def make_link(skill_dir, link):
    rel = os.path.relpath(skill_dir, os.path.dirname(link))
    try:
        os.symlink(rel, link, target_is_directory=True)
    except (OSError, NotImplementedError) as exc:
        winerror = getattr(exc, "winerror", None)
        hint = "--copy" if winerror == 1314 else "administrator/developer-mode privileges"
        raise RuntimeError("cannot create symlink %s (%s); rerun with %s" % (link, exc, hint))


def file_digest(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def collect_hashes(skill_dir, name):
    hashes = {}
    for dirpath, _dirnames, filenames in os.walk(skill_dir):
        for filename in filenames:
            full = os.path.join(dirpath, filename)
            rel = os.path.relpath(full, skill_dir).replace(os.sep, "/")
            hashes["%s/%s" % (name, rel)] = file_digest(full)
    return hashes


def read_copy_map():
    if not os.path.exists(COPY_MAP_PATH):
        return {}
    with open(COPY_MAP_PATH, "r") as handle:
        return json.load(handle).get("files", {})


def write_copy_map(hashes):
    payload = {"version": 1, "mode": "copy", "files": hashes}
    os.makedirs(os.path.dirname(COPY_MAP_PATH), exist_ok=True)
    with open(COPY_MAP_PATH, "w") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")


def check_copy(skills, skills_dir, recorded):
    findings = []
    for name, skill_dir in skills.items():
        live = collect_hashes(skill_dir, name)
        expected = dict((key, value) for key, value in recorded.items() if key.startswith(name + "/"))
        if not expected:
            findings.append((name, "copy-missing"))
            continue
        if live != expected:
            findings.append((name, "copy-drift"))
    known = set()
    for name in skills:
        known.add(name + "/")
    for key in recorded:
        if not any(key.startswith(prefix) for prefix in known):
            findings.append((key.split("/")[0], "copy-extra"))
    return findings


def check_target(skills, skills_dir, canonical_real, copy_mode):
    findings = []
    if copy_mode:
        for name, cls in check_copy(skills, skills_dir, read_copy_map()):
            findings.append((name, cls))
        return findings
    for name, skill_dir in sorted(skills.items()):
        cls = classify(os.path.join(skills_dir, name), skill_dir, canonical_real)
        if cls != OK:
            findings.append((name, cls))
    for entry, kind in extra_links(skills_dir, set(skills), canonical_real):
        findings.append((entry, EXTRA if kind == "link" else EXTRA_DIR))
    return findings


def do_check(targets, skills, canonical_real, force_copy, forbidden=()):
    failures = 0
    shadow = check_forbidden(forbidden)
    if shadow:
        print(" [ FIX ] %-12s shadow directory" % "forbidden")
        for rel, cls in shadow:
            print("        %-40s %s" % (rel, cls))
        print("        a skill under %s overrides the same-named skill in .agents/skills"
              % ", ".join(forbidden))
        print("        fix: delete it, or create that skill outside the host skills folder")
    for target in targets:
        label = target["label"]
        if not target.get("enabled", True):
            print(" [ OFF ] %-12s %s" % (label, target.get("notes", "disabled")))
            continue
        skills_dir = os.path.join(ROOT, target["skills_dir"])
        copy_mode = force_copy or target.get("mode") == "copy"
        if target.get("optional") and not os.path.isdir(skills_dir):
            # gitignored activation layer: absent means this host was never set
            # up here (a fresh clone, a CI runner). Not a defect (D50 B1).
            print(" [ SKIP ] %-12s %s (layer absent)" % (label, target["skills_dir"]))
            continue
        findings = check_target(skills, skills_dir, canonical_real, copy_mode)
        if not findings:
            print(" [ OK  ] %-12s %s" % (label, target["skills_dir"]))
            continue
        failures += 1
        print(" [ FIX ] %-12s %s" % (label, target["skills_dir"]))
        for name, cls in findings:
            print("        %-38s %s" % (name, cls))
    if failures or shadow:
        if failures:
            print("GATE-SKILLS ✗ %d target(s) need repair (run: sync)" % failures)
        if shadow:
            print("GATE-SKILLS ✗ %d shadowed skill(s) -- sync cannot fix this, delete them" % len(shadow))
        return 1
    print("GATE-SKILLS ✓ (%d skills × %d target(s))" % (len(skills), len(targets)))
    return 0


def do_sync(targets, skills, canonical_real, force_copy, dry_run):
    for target in targets:
        if not target.get("enabled", True):
            continue
        skills_dir = os.path.join(ROOT, target["skills_dir"])
        reasons = guard_reasons(skills_dir, canonical_real)
        if reasons:
            print("GATE-SKILLS ✗ guard %s: %s" % (target["skills_dir"], "; ".join(reasons)))
            return 2
    actions = 0
    for target in targets:
        label = target["label"]
        if not target.get("enabled", True):
            continue
        skills_dir = os.path.join(ROOT, target["skills_dir"])
        copy_mode = force_copy or target.get("mode") == "copy"
        tag = " [MKDIR] " if not os.path.isdir(skills_dir) else "        "
        print("%s%-12s %s" % (tag, label, target["skills_dir"]))
        if not dry_run:
            os.makedirs(skills_dir, exist_ok=True)
        if copy_mode:
            recorded = read_copy_map()
            for name, skill_dir in sorted(skills.items()):
                copy_path = os.path.join(skills_dir, name)
                live = collect_hashes(skill_dir, name)
                expected = dict((k, v) for k, v in recorded.items() if k.startswith(name + "/"))
                if os.path.isdir(copy_path) and not os.path.islink(copy_path) and live == expected:
                    continue
                actions += 1
                print(" [ COPY ] %-12s %s" % (label, name))
                if dry_run:
                    continue
                if os.path.islink(copy_path):
                    os.unlink(copy_path)
                elif os.path.isdir(copy_path):
                    shutil.rmtree(copy_path)
                shutil.copytree(skill_dir, copy_path)
                recorded.update(live)
            if not dry_run:
                write_copy_map(recorded)
            continue
        for name, skill_dir in sorted(skills.items()):
            link = os.path.join(skills_dir, name)
            cls = classify(link, skill_dir, canonical_real)
            if cls == OK:
                continue
            actions += 1
            if cls == EXISTS:
                backup = backup_name(skills_dir, name)
                print(" [BACKUP] %-12s %s -> %s" % (label, name, os.path.basename(backup)))
                if not dry_run:
                    shutil.move(link, backup)
                    prune_backups(skills_dir, name)
            elif os.path.islink(link):
                if not dry_run:
                    os.unlink(link)
            print(" [ LINK ] %-12s %s -> %s" % (label, name, os.path.relpath(skill_dir, skills_dir)))
            if not dry_run:
                make_link(skill_dir, link)
        for entry, kind in extra_links(skills_dir, set(skills), canonical_real):
            actions += 1
            path = os.path.join(skills_dir, entry)
            if kind == "dir":
                backup = backup_name(skills_dir, entry)
                print(" [BACKUP] %-12s %s -> %s" % (label, entry, os.path.basename(backup)))
                if not dry_run:
                    shutil.move(path, backup)
                    prune_backups(skills_dir, entry)
                continue
            print(" [ PRUNE] %-12s %s" % (label, entry))
            if not dry_run:
                os.unlink(path)
    if dry_run:
        print("%d action(s) would be applied (dry run)." % actions)
    else:
        print("GATE-SKILLS ✓ (%d action(s) applied)" % actions)
    return 0


def do_list(targets, skills):
    print("%-38s %s" % ("skill", "targets"))
    for name in sorted(skills):
        marks = []
        for target in targets:
            if not target.get("enabled", True):
                continue
            skills_dir = os.path.join(ROOT, target["skills_dir"])
            path = os.path.join(skills_dir, name)
            if os.path.islink(path) and os.path.exists(path):
                marks.append(target["label"])
            elif os.path.isdir(path) and not os.path.islink(path):
                marks.append(target["label"] + "(copy)")
        print("%-38s %s" % (name, " ".join(marks) or "-"))


def main(argv):
    if len(argv) < 2 or argv[1] not in ("check", "sync", "dry-run", "list"):
        print(__doc__.strip())
        return 0
    command = argv[1]
    force_copy = "--copy" in argv[2:]
    skills_root, targets, forbidden = load_config()
    skills = discover_skills(skills_root)
    canonical_real = os.path.realpath(skills_root)
    if command == "list":
        do_list(targets, skills)
        return 0
    if command == "check":
        return do_check(targets, skills, canonical_real, force_copy, forbidden)
    return do_sync(targets, skills, canonical_real, force_copy, dry_run=command == "dry-run")


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv))
    except RuntimeError as error:
        print("GATE-SKILLS ✗ %s" % error)
        sys.exit(1)
    except BrokenPipeError:
        os._exit(0)
