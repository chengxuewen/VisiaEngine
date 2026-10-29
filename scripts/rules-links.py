#!/usr/bin/env python3
"""Check / repair the per-host rules activation layer.

Single source: <repo>/.agents/rules/**/*.md -- the only editable copies.
Targets: declared in scripts/agent-targets.json (rules_dir per host).

Two modes, driven by each target's `rules_mode` (team-review #11, adjudicated 2026-10-28):
- `links` (default): transfer canonical `.md` as relative symlinks (or copies on Windows) verbatim — kept for hosts that read raw `.md`.
- `mdc`: generate CodeBuddy/Cursor-style `.mdc` — one subfolder per rule (`.codebuddy/rules/<lang>-<topic>/RULE.mdc`) with synthesized frontmatter (description/alwaysApply/enabled/updatedAt) and the canonical body verbatim (after stripping any canonical frontmatter), with ONE transform: cross-rule links `../<lang>/<topic>.md` are retargeted to the generated layout `../<lang>-<topic>/RULE.mdc` (a verbatim copy would point at a path that does not exist in the generated tree). Canonical is filtered by the target's `langs` allowlist. `gate-agents` compares regenerated content byte-for-byte (true-green).
- `copy` (2026-10-29, D50 Adj 1-B): generate plain `.md` project rules for a host with no frontmatter/alwaysApply switch (Trae). Only an explicit `rules_files` whitelist is transferred — `langs` cannot express "these exact files" — flattened to `<lang>-<topic>.md` in ONE directory so every rule stays project-wide. Frontmatter is stripped and cross-rule links are flattened to siblings. When `router_card` is set, a synthesized `00-router.md` indexes the canonical tree with per-language triggers; it is generated, never canonical prose, because canonical files are mirrored into the other hosts (R2).

Exit codes: 0 clean, 1 findings, 2 guard violation (nothing written).
Reuses scripts/agent_common.py (guard / classify / link / backup).
"""

import os
import sys

import agent_common as ac

PREFIX = "GATE-RULES"

import hashlib
import re
import shutil

_FM = re.compile(r"^---\n.*?\n---\n", re.DOTALL)


def _langs(target):
    langs = target.get("langs")
    return langs if isinstance(langs, list) and langs else None


def _canonical_rels(rules_root, langs):
    rels = ac.iter_rel_files(rules_root, ".md")
    out = []
    for rel in rels:
        parts = rel.split("/")
        if len(parts) != 2:
            continue  # README.md at root, or unexpected depth
        if langs is not None and parts[0] not in langs:
            continue
        out.append(rel)
    return sorted(out)


def _strip_frontmatter(text):
    m = _FM.match(text)
    return text[m.end():] if m else text


def _first_h1(body):
    for line in body.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return None


def _mdc_folder(rel):
    lang, topic = rel.split("/")
    if topic.endswith(".md"):
        topic = topic[:-3]
    return "%s-%s" % (lang, topic)


# Cross-rule links point at the canonical layout (`../common/testing.md`); the
# generated tree uses one folder per rule (`common-testing/RULE.mdc`). This is the
# only non-verbatim transform applied to the canonical body.
_LINK = re.compile(r"\]\(\.\./([a-z0-9-]+)/([a-z0-9-]+)\.md((?:#[^)]*)?)\)")


def _retarget_links(body):
    return _LINK.sub(lambda m: "](../%s-%s/RULE.mdc%s)" % (m.group(1), m.group(2), m.group(3)), body)


def _mdc_content(canonical_path, rel, always_on=None, descriptions=None):
    with open(canonical_path, "r", encoding="utf-8") as handle:
        raw = handle.read()
    body = _retarget_links(_strip_frontmatter(raw))
    h1 = _first_h1(body)
    folder = _mdc_folder(rel)
    if h1 is None:
        h1 = folder
    desc_text = h1
    if descriptions and folder in descriptions:
        desc_text = descriptions[folder]
    desc = '"%s"' % desc_text.replace("\\", "\\\\").replace('"', '\\"')
    digest = hashlib.sha256(raw.encode("utf-8")).hexdigest()[:12]
    apply = "true" if (always_on and folder in always_on) else "false"
    front = "---\ndescription: %s\nalwaysApply: %s\nenabled: true\nupdatedAt: %s\n---\n" % (desc, apply, digest)
    return folder, front + body


def sync_layer_mdc(target, rules_root, dry_run):
    target_dir = _resolve(target)
    if target_dir is None:
        return 0
    always_on = set(target.get("always_on_rules") or [])
    descriptions = target.get("rule_descriptions") or {}
    rels = _canonical_rels(rules_root, _langs(target))
    ac.ensure_dir(target_dir)
    actions = 0
    expected = set()
    for rel in rels:
        canonical = os.path.join(rules_root, rel)
        folder, content = _mdc_content(canonical, rel, always_on, descriptions)
        expected.add(folder)
        rfile = os.path.join(target_dir, folder, "RULE.mdc")
        current = None
        if os.path.isfile(rfile):
            with open(rfile, "r", encoding="utf-8") as handle:
                current = handle.read()
        if current == content:
            continue
        actions += 1
        print(" [ MDCMK] %-10s %s" % (target["label"], folder))
        if not dry_run:
            ac.ensure_dir(os.path.join(target_dir, folder))
            with open(rfile, "w", encoding="utf-8") as handle:
                handle.write(content)
    if os.path.isdir(target_dir):
        for entry in sorted(os.listdir(target_dir)):
            if entry in expected:
                continue
            full = os.path.join(target_dir, entry)
            actions += 1
            print(" [ MDCDEL] %-9s %s" % (target["label"], entry))
            if dry_run:
                continue
            if os.path.islink(full):
                os.unlink(full)
            elif os.path.isdir(full):
                shutil.rmtree(full, ignore_errors=True)
            else:
                backup = ac.backup_name(target_dir, entry)
                os.rename(full, backup)
                ac.prune_backups(target_dir, entry)
    return actions


def check_layer_mdc(target, rules_root):
    target_dir = _resolve(target)
    if target_dir is None:
        return []
    always_on = set(target.get("always_on_rules") or [])
    descriptions = target.get("rule_descriptions") or {}
    findings = []
    if not os.path.isdir(target_dir):
        if os.path.isdir(rules_root):
            findings.append(("__layer__", ac.MISSING))
        return findings
    rels = _canonical_rels(rules_root, _langs(target))
    expected = set()
    for rel in rels:
        canonical = os.path.join(rules_root, rel)
        folder, content = _mdc_content(canonical, rel, always_on, descriptions)
        expected.add(folder)
        rfile = os.path.join(target_dir, folder, "RULE.mdc")
        if not os.path.isfile(rfile):
            findings.append((rel, "missing"))
            continue
        with open(rfile, "r", encoding="utf-8") as handle:
            actual = handle.read()
        if actual != content:
            findings.append((rel, "mismatch"))
    for entry in sorted(os.listdir(target_dir)):
        if ".bak-" in entry:
            continue  # a deliberate backup from the last prune, not drift
        if entry not in expected:
            findings.append((entry, "extra"))
    return findings


def _resolve(target):
    rules_dir = target.get("rules_dir")
    if not rules_dir:
        return None
    return os.path.join(ac.ROOT, rules_dir)


# --- copy mode (plain `.md` project rules: no frontmatter, no globs, no links) --
# Trae reads `.trae/rules/*.md` as project-wide rules and has no alwaysApply switch,
# so "which files exist" IS the budget decision (D50 Adj 1-B). The whitelist is an
# explicit file list because `langs` means "language folders", not "these files".
# The router card is SYNTHESIZED from the tree rather than kept as canonical prose:
# canonical files are mirrored into every other host, and a card naming host-specific
# paths would turn up in CodeBuddy's tree as a rule that makes no sense there (R2).

ROUTER_CARD = "00-router.md"

_LANG_TRIGGER = {
    "rust": "Rust work (.rs)",
    "cpp": "C / C++ work (.h, .hpp, .cpp)",
    "typescript": "TypeScript work (.ts)",
    "python": "Python work (.py)",
    "web": "web page work (HTML / CSS / JS)",
    "common": "language-independent work and process questions",
}


def _flat_name(rel):
    """`common/constraints.md` -> `common-constraints` (flat folder, project-wide)."""
    parts = rel.split("/")
    if len(parts) != 2:
        return None
    lang, topic = parts
    return "%s-%s" % (lang, topic[:-3] if topic.endswith(".md") else topic)


def _copy_body(rules_root, rel):
    """Canonical body, frontmatter stripped, cross-rule links flattened to siblings."""
    with open(os.path.join(rules_root, rel), "r", encoding="utf-8") as handle:
        body = _strip_frontmatter(handle.read())
    return _LINK.sub(lambda m: "](%s-%s.md%s)" % (m.group(1), m.group(2), m.group(3)), body)


def _whitelist(target, rules_root):
    present, missing = [], []
    for name in target.get("rules_files") or []:
        rel = name.strip()
        flat = _flat_name(rel)
        if not rel.endswith(".md") or flat is None:
            missing.append(rel)  # unusable entry, reported rather than silently dropped
            continue
        (present if os.path.isfile(os.path.join(rules_root, rel)) else missing).append(rel)
    return sorted(present), sorted(missing)


def _router_text(rules_root, whitelist):
    tree = {}
    for rel in _canonical_rels(rules_root, None):
        lang, topic = rel.split("/")
        tree.setdefault(lang, []).append(topic[:-3])
    lines = [
        "# Project rules index",
        "",
        "Canonical rules live in `.agents/rules/<lang>/<topic>.md`.",
        "These two are copied next to this card so they are always in effect:",
        "",
    ]
    for rel in whitelist:
        lines.append("- `%s`" % rel)
    lines += ["", "Read one more when its trigger applies:", ""]
    for lang in sorted(tree):
        trigger = _LANG_TRIGGER.get(lang, "%s-specific work" % lang)
        lines.append("- %s -> `%s/`: %s" % (trigger, lang, ", ".join(sorted(tree[lang]))))
    return "\n".join(lines).rstrip("\n") + "\n"


def _expected_copy_tree(target, rules_root):
    whitelist, missing = _whitelist(target, rules_root)
    expected = {}
    for rel in whitelist:
        expected[_flat_name(rel) + ".md"] = _copy_body(rules_root, rel)
    if target.get("router_card"):
        expected[ROUTER_CARD] = _router_text(rules_root, whitelist)
    return expected, missing


def sync_layer_copy(target, rules_root, dry_run):
    target_dir = _resolve(target)
    if target_dir is None:
        return 0
    expected, missing = _expected_copy_tree(target, rules_root)
    for rel in missing:
        print(" [ WARN ] %-10s whitelist entry unusable: %s" % (target["label"], rel))
    ac.ensure_dir(target_dir)
    actions = 0
    for name, content in sorted(expected.items()):
        path = os.path.join(target_dir, name)
        current = None
        if os.path.isfile(path):
            with open(path, "r", encoding="utf-8") as handle:
                current = handle.read()
        if current == content:
            continue
        actions += 1
        print(" [ COPY ] %-10s %s" % (target["label"], name))
        if not dry_run:
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(content)
    if missing:
        # A whitelist entry is unusable, so the expected tree is incomplete.
        # Pruning now would delete files that are still wanted: leave it alone.
        print(" [ WARN ] %-10s prune skipped (unusable whitelist entries)" % target["label"])
        return actions
    for entry in sorted(os.listdir(target_dir)):
        if entry in expected:
            continue
        actions += 1
        print(" [ PRUNE] %-10s %s" % (target["label"], entry))
        if dry_run:
            continue
        full = os.path.join(target_dir, entry)
        if os.path.islink(full):
            os.unlink(full)
        elif os.path.isdir(full):
            shutil.rmtree(full, ignore_errors=True)  # the host may nest rule folders
        else:
            backup = ac.backup_name(target_dir, entry)
            os.rename(full, backup)
            ac.prune_backups(target_dir, entry)
    return actions


def check_layer_copy(target, rules_root):
    target_dir = _resolve(target)
    if target_dir is None:
        return []
    expected, missing = _expected_copy_tree(target, rules_root)
    findings = [(rel, "whitelist-unusable") for rel in missing]
    for name, content in sorted(expected.items()):
        path = os.path.join(target_dir, name)
        if not os.path.isfile(path):
            findings.append((name, "missing"))
            continue
        with open(path, "r", encoding="utf-8") as handle:
            if handle.read() != content:
                findings.append((name, "mismatch"))
    for entry in sorted(os.listdir(target_dir)):
        if entry in expected:
            continue
        if ".bak-" in entry:
            continue  # a deliberate backup from the last prune, not drift
        cls = "subdir" if os.path.isdir(os.path.join(target_dir, entry)) else "extra"
        findings.append((entry, cls))
    return findings


def check_layer(target, rules_root, canonical_real, copy_mode):
    target_dir = _resolve(target)
    if target_dir is None:
        return []
    findings = []
    if not os.path.isdir(target_dir):
        # missing whole layer is a finding only if canonical has content
        if os.path.isdir(rules_root):
            findings.append(("__layer__", ac.MISSING))
        return findings
    rels = ac.iter_rel_files(rules_root, ".md")
    for rel in rels:
        link = os.path.join(target_dir, rel)
        src = os.path.realpath(os.path.join(rules_root, rel))
        cls = ac.classify_link(link, src)
        if cls != ac.OK:
            findings.append((rel, cls))
    # stale: .md in target not present in canonical
    for rel in ac.iter_rel_files(target_dir, ".md"):
        if rel not in set(rels):
            findings.append((rel, "extra"))
    return findings


def sync_layer(target, rules_root, canonical_real, copy_mode, dry_run):
    target_dir = _resolve(target)
    if target_dir is None:
        return 0
    actions = 0
    rels = set(ac.iter_rel_files(rules_root, ".md"))
    ac.ensure_dir(target_dir)
    for rel in sorted(rels):
        src = os.path.join(rules_root, rel)
        link = os.path.join(target_dir, rel)
        src_real = os.path.realpath(src)
        cls = ac.classify_link(link, src_real)
        if cls == ac.OK:
            continue
        actions += 1
        ac.ensure_dir(os.path.dirname(link))
        if cls == ac.EXISTS:
            backup = ac.backup_name(target_dir, rel)
            print(" [BACKUP] %-10s %s -> %s" % (target["label"], rel, os.path.basename(backup)))
            if not dry_run:
                shutil_move(link, backup)
                ac.prune_backups(target_dir, rel)
        elif os.path.islink(link):
            if not dry_run:
                os.unlink(link)
        print(" [ LINK ] %-10s %s -> %s" % (target["label"], rel, os.path.relpath(src, target_dir)))
        if not dry_run:
            try:
                ac.make_link(src, link)
            except RuntimeError:
                if not copy_mode:
                    raise
                ac.copy_tree(src, link)
    # prune stale
    if os.path.isdir(target_dir):
        for rel in ac.iter_rel_files(target_dir, ".md"):
            if rel not in rels:
                actions += 1
                path = os.path.join(target_dir, rel)
                print(" [ PRUNE] %-10s %s" % (target["label"], rel))
                if not dry_run:
                    if os.path.islink(path):
                        os.unlink(path)
                    else:
                        ac.backup_name(target_dir, rel)  # leave real file as backup
    return actions


def shutil_move(src, dst):
    import shutil
    shutil.move(src, dst)


def run(command, targets, cfg, dry_run):
    rules_root = os.path.join(ac.ROOT, cfg["rules_root"])
    canonical_real = os.path.realpath(rules_root)
    if not os.path.isdir(rules_root):
        print("%s ✗ canonical-empty %s" % (PREFIX, rules_root))
        return ac.EXIT_GUARD
    active = [t for t in targets if t.get("enabled") and _resolve(t)]
    if command == "list":
        for t in targets:
            if _resolve(t):
                print("%-10s rules %s" % (t["label"], t.get("rules_dir")))
        return ac.EXIT_OK
    if command == "check":
        failures = 0
        total = 0
        for t in active:
            if t.get("optional") and not os.path.isdir(_resolve(t)):
                # gitignored activation layer: absent means this host was never
                # set up here (a fresh clone, a CI runner). Not a defect (D50 B1).
                print(" [ SKIP ] %-10s %s (layer absent)" % (t["label"], t.get("rules_dir")))
                continue
            reasons = ac.guard_reasons(_resolve(t), canonical_real)
            if reasons:
                ac.die_guard(PREFIX, t.get("rules_dir"), reasons)
            if t.get("rules_mode") == "mdc":
                findings = check_layer_mdc(t, rules_root)
                total += len(_canonical_rels(rules_root, _langs(t)))
            elif t.get("rules_mode") == "copy":
                findings = check_layer_copy(t, rules_root)
                total += len(_expected_copy_tree(t, rules_root)[0])
            else:
                findings = check_layer(t, rules_root, canonical_real, False)
                total += len(ac.iter_rel_files(rules_root, ".md"))
            if not findings:
                print(" [ OK  ] %-10s %s" % (t["label"], t.get("rules_dir")))
                continue
            failures += 1
            print(" [ FIX ] %-10s %s" % (t["label"], t.get("rules_dir")))
            for rel, cls in findings:
                print("        %-40s %s" % (rel, cls))
        if failures:
            print("%s ✗ %d target(s) need repair (run: agent-links.py sync)" % (PREFIX, failures))
            return ac.EXIT_FINDINGS
        print("%s ✓ (%d rule files × %d target(s))" % (PREFIX, total, len(active)))
        return ac.EXIT_OK
    # sync / dry-run
    for t in active:
        reasons = ac.guard_reasons(_resolve(t), canonical_real)
        if reasons:
            ac.die_guard(PREFIX, t.get("rules_dir"), reasons)
        print("%s %-10s %s" % ("        " if not dry_run else " [DRY] ", t["label"], t.get("rules_dir")))
        if t.get("rules_mode") == "mdc":
            sync_layer_mdc(t, rules_root, dry_run)
        elif t.get("rules_mode") == "copy":
            sync_layer_copy(t, rules_root, dry_run)
        else:
            sync_layer(t, rules_root, canonical_real, copy_mode=False, dry_run=dry_run)
    print("%s ✓ sync done" % PREFIX)
    return ac.EXIT_OK


if __name__ == "__main__":
    print(__doc__.strip())
    sys.exit(0)
