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
import tomllib

root = Path("docs")
paths = [Path("README.md"), Path("CONTRIBUTING.md"), Path("SECURITY.md")]
paths.extend(sorted(root.rglob("*.md")))
errors = []
link_pattern = re.compile(r"!?\[[^]]*\]\(([^)]+)\)")

with Path("zensical.toml").open("rb") as handle:
    site_config = tomllib.load(handle)

nav = site_config["project"]["nav"]

def nav_paths(entries):
    for entry in entries:
        for value in entry.values():
            if isinstance(value, str):
                yield value
            elif isinstance(value, list):
                yield from nav_paths(value)

for nav_path in nav_paths(nav):
    path = root / nav_path
    if not path.is_file():
        errors.append(f"zensical.toml: navigation path does not exist: {nav_path}")

for path in root.glob("*.md"):
    if path.name != path.name.lower():
        errors.append(f"{path}: root documentation filenames must be lowercase")

package_config = tomllib.loads(
    Path("crates/taytay/Cargo.toml").read_text(encoding="utf-8")
)
crate_version = package_config["package"]["version"]
dependency_pattern = re.compile(r"lunsaran-taytay\s*=\s*\"([^\"]+)\"")
for path in (Path("README.md"), Path("docs/quickstart.md")):
    match = dependency_pattern.search(path.read_text(encoding="utf-8"))
    if not match or match.group(1) != crate_version:
        errors.append(
            f"{path}: documented lunsaran-taytay version does not match {crate_version}"
        )

for path in paths:
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

print(f"Documentation lint passed for {len(paths)} Markdown files")
PY
