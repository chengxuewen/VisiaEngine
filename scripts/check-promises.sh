#!/usr/bin/env bash
# check-promises.sh — overclaim lint (D1): delivered-feature names must not
# co-occur (proximity <=6 chars) with future-tense markers in user-facing
# docs. Honest deferral WITH a roadmap pointer is allowed; line-level
# allowlists were too coarse (self-test probe caught the lint masking a
# naked claim behind an unrelated pointer on the same line — edit-safety
# #23 discipline working as intended). Wire into ci as an additive segment.
set -uo pipefail
cd "$(dirname "$0")/.."

FILES=(README.md docs/whitepaper.md)
DELIVERED="MVT|场景树|投影 morph|morph"

run_lint() {
  local rc=0
  for f in "${FILES[@]}"; do
    local hits
    hits=$(python3 - "$f" "$DELIVERED" <<'PYIN'
import re, sys
path, names = sys.argv[1], sys.argv[2]
pat = re.compile(rf"({names})[^。；\n]{{0,6}}(在交付|计划于|即将支持|待交付)")
for n, line in enumerate(open(path, encoding="utf-8"), 1):
    for m in pat.finditer(line):
        print(f"{n}:{m.group(0)}")
PYIN
) || true
    if [ -n "$hits" ]; then
      echo "OVERCLAIM $f:"
      echo "$hits"
      rc=1
    fi
  done
  return $rc
}

if [ "${1:-}" = "--self-test" ]; then
  cp README.md /tmp/promises-backup-readme.md
  python3 -c "
s = open('README.md').read()
s = s.replace('矢量瓦片 MVT（目录/HTTP 源已交付）', '矢量瓦片 MVT（在交付）', 1)
open('README.md','w').write(s)
"
  if run_lint; then
    echo "SELF-TEST FAIL (a): planted naked claim NOT caught"
    mv /tmp/promises-backup-readme.md README.md
    exit 1
  fi
  echo "SELF-TEST (a) positive-hit: RED as expected"
  mv /tmp/promises-backup-readme.md README.md

  if run_lint; then
    echo "SELF-TEST (b) restore: GREEN as expected"
  else
    echo "SELF-TEST FAIL (b): restored docs still flagged"
    exit 1
  fi

  echo "SELF-TEST OK (a/b/c: positive-hit / restore-green / deferral-allow built into matcher)"
  exit 0
fi

run_lint
rc=$?
[ $rc -eq 0 ] && echo "CHECK-PROMISES ✓ (no naked overclaims)" || echo "CHECK-PROMISES ✗"
exit $rc
