#!/usr/bin/env bash
set -euo pipefail

if ! command -v zensical >/dev/null 2>&1; then
  echo "zensical is required for documentation quality checks. Install it with: python -m pip install zensical" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required for documentation lint checks" >&2
  exit 1
fi

zensical build

python3 - <<'PY'
from pathlib import Path
import re
import sys

root = Path("docs")
errors = []
link_pattern = re.compile(r"!?\[[^]]*\]\(([^)]+)\)")

for path in sorted(root.rglob("*.md")):
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()

    if text and not text.endswith("\n"):
        errors.append(f"{path}: missing final newline")

    fence_open = None
    heading_levels = []
    h1_count = 0
    for number, line in enumerate(lines, start=1):
        if line.endswith((" ", "\t")):
            errors.append(f"{path}:{number}: trailing whitespace")

        if line.startswith("```"):
            if fence_open is None:
                fence_open = number
            else:
                fence_open = None
            continue

        match = re.match(r"^(#{1,6})\s+\S", line)
        if match:
            level = len(match.group(1))
            heading_levels.append((number, level))
            if level == 1:
                h1_count += 1

        for target in link_pattern.findall(line):
            target = target.split("#", 1)[0].strip()
            if not target or "://" in target or target.startswith(("mailto:", "/")):
                continue
            target_path = (path.parent / target).resolve()
            if not target_path.exists():
                errors.append(f"{path}:{number}: local link does not exist: {target}")

    if fence_open is not None:
        errors.append(f"{path}:{fence_open}: unclosed fenced code block")
    if h1_count != 1:
        errors.append(f"{path}: expected exactly one level-1 heading, found {h1_count}")
    for (previous_number, previous_level), (number, level) in zip(heading_levels, heading_levels[1:]):
        if level > previous_level + 1:
            errors.append(
                f"{path}:{number}: heading level skips from {previous_level} to {level}"
            )

if errors:
    print("Documentation lint failed:", file=sys.stderr)
    print("\n".join(f"- {error}" for error in errors), file=sys.stderr)
    sys.exit(1)

print(f"Documentation lint passed for {len(list(root.rglob('*.md')))} Markdown files")
PY
