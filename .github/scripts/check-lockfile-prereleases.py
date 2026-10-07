#!/usr/bin/env python3
"""Fail CI if Cargo.lock contains unexpected prerelease crate versions.

Accepted until ssh-key 0.7.0 is stable (issue #64): russh exact-pins
ssh-key 0.7.0-rc.*, and no stable 0.7.x exists on crates.io. argon2 0.6
and blake2 0.11 are already stable; they must not reappear as RCs.

Build metadata after '+' is ignored so WASI versions such as
`0.4.0+wasi-0.3.0-rc-...` are not treated as Cargo prereleases.

Exit condition: ssh-key 0.7.0 stable lands AND russh depends on it.
Then delete the allowlist below, drop this job if it becomes a no-op,
and close #64.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ALLOWED_PRERELEASES = {
    # russh 0.63.x and 0.64.x still exact-pin this RC. A newer 0.7.0-rc.N
    # is the same line; a different prerelease line is not.
    "ssh-key": re.compile(r"^0\.7\.0-rc\.\d+$"),
}

PACKAGE_NAME = re.compile(r'^name = "([^"]+)"$', re.M)
PACKAGE_VERSION = re.compile(r'^version = "([^"]+)"$', re.M)


def cargo_core_version(version: str) -> str:
    """Strip Cargo build metadata (`+...`)."""
    return version.split("+", 1)[0]


def is_prerelease(version: str) -> bool:
    return "-" in cargo_core_version(version)


def packages_from_lock(text: str) -> list[tuple[str, str]]:
    packages: list[tuple[str, str]] = []
    for block in text.split("[[package]]"):
        name_match = PACKAGE_NAME.search(block)
        version_match = PACKAGE_VERSION.search(block)
        if name_match and version_match:
            packages.append((name_match.group(1), version_match.group(1)))
    return packages


def check_lock(text: str, label: str, *, warn_unused: bool = True) -> list[str]:
    errors: list[str] = []
    seen_allowed: set[str] = set()
    for name, version in packages_from_lock(text):
        core = cargo_core_version(version)
        if not is_prerelease(version):
            continue
        pattern = ALLOWED_PRERELEASES.get(name)
        if pattern is not None and pattern.fullmatch(core):
            seen_allowed.add(name)
            continue
        errors.append(f"{label}: unexpected prerelease {name} {version}")
    if warn_unused:
        for name in ALLOWED_PRERELEASES:
            if name not in seen_allowed:
                print(
                    f"{label}: allowlist unused for {name}; "
                    "if ssh-key 0.7.0 is stable and russh depends on it, "
                    "remove the allowlist and close #64",
                    file=sys.stderr,
                )
    return errors


def find_lockfiles(root: Path) -> list[Path]:
    lockfiles: list[Path] = []
    for path in sorted(root.rglob("Cargo.lock")):
        parts = set(path.parts)
        if parts & {"target", ".git", "vendor"}:
            continue
        lockfiles.append(path)
    return lockfiles


def self_test() -> int:
    cases: list[tuple[str, str, bool]] = [
        (
            "allowed ssh-key rc",
            '[[package]]\nname = "ssh-key"\nversion = "0.7.0-rc.11"\n',
            True,
        ),
        (
            "argon2 rc is not allowed",
            '[[package]]\nname = "argon2"\nversion = "0.6.0-rc.8"\n',
            False,
        ),
        (
            "wasi build metadata is not a prerelease",
            '[[package]]\nname = "wasip3"\nversion = "0.4.0+wasi-0.3.0-rc-2026-01-06"\n',
            True,
        ),
        (
            "different ssh-key prerelease line is not allowed",
            '[[package]]\nname = "ssh-key"\nversion = "0.8.0-rc.1"\n',
            False,
        ),
        (
            "stable tree is fine",
            '[[package]]\nname = "ssh-key"\nversion = "0.7.0"\n',
            True,
        ),
        (
            "blake2 rc is not allowed",
            '[[package]]\nname = "blake2"\nversion = "0.11.0-rc.6"\n',
            False,
        ),
    ]
    failed = 0
    for title, text, expect_ok in cases:
        errors = check_lock(text, title, warn_unused=False)
        ok = not errors
        if ok != expect_ok:
            failed += 1
            print(f"SELFTEST FAIL: {title}: ok={ok} expected={expect_ok} errors={errors}")
        else:
            print(f"SELFTEST OK: {title}")
    return 1 if failed else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path("."),
        help="repository root to scan for Cargo.lock files",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run built-in fixtures and exit",
    )
    args = parser.parse_args()
    if args.self_test:
        return self_test()

    root = args.root.resolve()
    lockfiles = find_lockfiles(root)
    if not lockfiles:
        print(f"no Cargo.lock files under {root}", file=sys.stderr)
        return 1

    errors: list[str] = []
    for lock in lockfiles:
        rel = lock.relative_to(root)
        errors.extend(check_lock(lock.read_text(encoding="utf-8"), str(rel)))
        print(f"checked {rel}")

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print("prerelease guard: only allowlisted ssh-key 0.7.0-rc.* present")
    return 0


if __name__ == "__main__":
    sys.exit(main())
