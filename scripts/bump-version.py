#!/usr/bin/env python3
"""
Automate release version bumping (Android + iOS + git tag).

Purpose:
- keep release version values aligned in:
  - vault-android/app/build.gradle.kts
  - vault-ios/Vault.xcodeproj/project.pbxproj
- keep git release tag vX.Y.Z aligned with app version
- support retry of same semantic version with higher build after review rejection

Usage:
- scripts/bump-version.py [--version X.Y.Z] [--build N] [--retag] [--no-commit] [--no-tag]
- if --version is omitted: resolve next patch from remote tags on origin
- default build: 1

Build code formula:
- build_code = major*100000000 + minor*100000 + patch*1000 + build
- example: 0.1.23 + build 2 -> 123002

Script behavior:
- updates Android: versionCode, versionName
- updates iOS (after marker IDs for Debug + Release):
  - CURRENT_PROJECT_VERSION
  - MARKETING_VERSION
- stages both files
- optionally creates commit: "Bump version"
- optionally creates tag: vX.Y.Z

Safety rules:
- never pushes anything
- fails if remote tag already exists
- fails if local tag exists unless --retag
- --retag is local-only and allowed only when remote tag does not exist
- when commit is enabled, fails if anything is already staged

Rejected app store review workflow:
1) remove local unpushed "Bump version" commit
2) implement fixes
3) rerun with same --version and higher --build, usually with --retag
4) rebuild and resubmit
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path
from typing import Iterable, List, Optional, Tuple

REPO_ROOT = Path(__file__).resolve().parents[1]
ANDROID_PROJECT_FILE = REPO_ROOT / "vault-android/app/build.gradle.kts"
IOS_PROJECT_FILE = REPO_ROOT / "vault-ios/Vault.xcodeproj/project.pbxproj"
IOS_MARKERS = [
    "A18A8CF52917988200B8F8B9",  # Debug
    "A18A8CF62917988200B8F8B9",  # Release
]


class BumpError(Exception):
    pass


def run_git(args: List[str], check: bool = True) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        ["git", *args],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
    )
    if check and proc.returncode != 0:
        stderr = proc.stderr.strip()
        stdout = proc.stdout.strip()
        details = stderr or stdout or f"git {' '.join(args)} failed"
        raise BumpError(details)
    return proc


def parse_version(version: str) -> Tuple[int, int, int]:
    if version.startswith("v"):
        version = version[1:]
    m = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)", version)
    if not m:
        raise BumpError(
            f"Invalid --version '{version}'. Expected numeric semver like 0.1.23"
        )
    return tuple(int(x) for x in m.groups())


def get_build_code(version: str, build: int) -> int:
    major, minor, patch = parse_version(version)
    # Keep build as the 3-digit suffix: MMMMMmmmpppbbb via weighted decimal
    # slots.
    return major * 100_000_000 + minor * 100_000 + patch * 1_000 + build


def remote_tag_exists(tag: str) -> bool:
    proc = run_git(["ls-remote", "--tags", "origin", f"refs/tags/{tag}"], check=False)
    if proc.returncode != 0:
        raise BumpError(
            "Failed to query remote tags via git ls-remote. "
            "Check network/auth/remote configuration and retry."
        )
    return bool(proc.stdout.strip())


def local_tag_exists(tag: str) -> bool:
    proc = run_git(["show-ref", "--verify", f"refs/tags/{tag}"], check=False)
    return proc.returncode == 0


def list_remote_semver_tags() -> List[Tuple[int, int, int]]:
    proc = run_git(["ls-remote", "--tags", "origin", "refs/tags/v*"])
    versions: set[Tuple[int, int, int]] = set()
    for raw in proc.stdout.splitlines():
        raw = raw.strip()
        if not raw:
            continue
        parts = raw.split("\t", 1)
        if len(parts) != 2:
            continue
        ref = parts[1]
        if ref.endswith("^{}"):
            continue
        if not ref.startswith("refs/tags/v"):
            continue
        tag = ref.removeprefix("refs/tags/v")
        m = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)", tag)
        if not m:
            continue
        versions.add(tuple(int(x) for x in m.groups()))
    return sorted(versions)


def resolve_version(explicit: Optional[str]) -> str:
    if explicit:
        parse_version(explicit)
        return explicit

    versions = list_remote_semver_tags()
    if not versions:
        raise BumpError(
            "No remote release tags like vX.Y.Z were found. "
            "Provide --version explicitly."
        )
    major, minor, patch = versions[-1]
    return f"{major}.{minor}.{patch + 1}"


def ensure_commit_preflight(no_commit: bool) -> None:
    if no_commit:
        return
    proc = run_git(["diff", "--cached", "--name-only"])
    if proc.stdout.strip():
        raise BumpError(
            "Refusing to run because staged files already exist. "
            "When commit is enabled, the staging area must be empty to avoid "
            "committing unrelated changes."
        )


def update_android_project_file(target_version: str, target_build_code: int) -> None:
    text = ANDROID_PROJECT_FILE.read_text()
    lines = text.splitlines(keepends=True)

    start_idx = None
    for idx, line in enumerate(lines):
        if re.search(r"^\s*defaultConfig\s*\{\s*$", line):
            start_idx = idx
            break
    if start_idx is None:
        raise BumpError("Could not find defaultConfig block in Android Gradle file")

    depth = 0
    end_idx = None
    for idx in range(start_idx, len(lines)):
        depth += lines[idx].count("{")
        depth -= lines[idx].count("}")
        if idx > start_idx and depth == 0:
            end_idx = idx
            break
    if end_idx is None:
        raise BumpError("Could not determine end of defaultConfig block")

    changed_code = 0
    changed_name = 0
    for idx in range(start_idx, end_idx + 1):
        line = lines[idx]
        content = line.rstrip("\n")

        code_match = re.match(r"^(\s*versionCode = )(\d+)$", content)
        if code_match and changed_code == 0:
            lines[idx] = f"{code_match.group(1)}{target_build_code}\n"
            changed_code += 1
            continue

        name_match = re.match(r'^(\s*versionName = ")([^"]+)(")$', content)
        if name_match and changed_name == 0:
            lines[idx] = f"{name_match.group(1)}{target_version}{name_match.group(3)}\n"
            changed_name += 1

    if changed_code != 1 or changed_name != 1:
        raise BumpError(
            "Failed to update Android version fields exactly once in defaultConfig "
            f"(versionCode changes={changed_code}, versionName changes={changed_name})."
        )

    ANDROID_PROJECT_FILE.write_text("".join(lines))


def _find_first_after(
    lines: List[str], start: int, pattern: re.Pattern[str]
) -> Optional[int]:
    for idx in range(start + 1, len(lines)):
        if pattern.search(lines[idx]):
            return idx
    return None


def update_ios_project_file(target_version: str, target_build_code: int) -> None:
    text = IOS_PROJECT_FILE.read_text()
    lines = text.splitlines(keepends=True)

    current_pattern = re.compile(r"^(\s*CURRENT_PROJECT_VERSION = )(\d+);$")
    marketing_pattern = re.compile(r"^(\s*MARKETING_VERSION = )(\d+\.\d+\.\d+);$")

    changed = 0
    for marker in IOS_MARKERS:
        marker_idx = next(
            (i for i, line in enumerate(lines) if marker in line and " = {" in line),
            None,
        )
        if marker_idx is None:
            raise BumpError(f"Could not find iOS marker {marker}")

        current_idx = _find_first_after(lines, marker_idx, current_pattern)
        if current_idx is None:
            raise BumpError(
                f"Could not find CURRENT_PROJECT_VERSION after marker {marker}"
            )
        marketing_idx = _find_first_after(lines, marker_idx, marketing_pattern)
        if marketing_idx is None:
            raise BumpError(f"Could not find MARKETING_VERSION after marker {marker}")

        current_line = lines[current_idx]
        marketing_line = lines[marketing_idx]
        current_content = current_line.rstrip("\n")
        marketing_content = marketing_line.rstrip("\n")

        c = current_pattern.match(current_content)
        m = marketing_pattern.match(marketing_content)
        if c is None or m is None:
            raise BumpError("Internal parsing error while editing iOS project file")

        lines[current_idx] = f"{c.group(1)}{target_build_code};\n"
        lines[marketing_idx] = f"{m.group(1)}{target_version};\n"
        changed += 2

    if changed != 4:
        raise BumpError(f"Expected 4 iOS changes, got {changed}")

    IOS_PROJECT_FILE.write_text("".join(lines))


def git_add_version_files() -> None:
    run_git(
        [
            "add",
            str(ANDROID_PROJECT_FILE.relative_to(REPO_ROOT)),
            str(IOS_PROJECT_FILE.relative_to(REPO_ROOT)),
        ]
    )


def commit_if_enabled(no_commit: bool) -> str:
    if no_commit:
        return "skipped"
    run_git(["commit", "-m", "Bump version"])
    return "created"


def preflight_tag_policy(tag: str, no_tag: bool, retag: bool) -> None:
    if no_tag:
        return

    if remote_tag_exists(tag):
        raise BumpError(
            f"Remote tag {tag} already exists on origin. "
            "Refusing to continue because pushed tags are immutable in this workflow."
        )

    has_local = local_tag_exists(tag)
    if retag and not has_local:
        raise BumpError(f"--retag was provided but local tag {tag} does not exist.")
    if not retag and has_local:
        raise BumpError(
            f"Local tag {tag} already exists. Use --retag to recreate it locally "
            "(allowed only when remote tag does not exist)."
        )


def apply_tag(tag: str, no_tag: bool, retag: bool) -> str:
    if no_tag:
        return "skipped"

    if retag:
        run_git(["tag", "-d", tag])
        run_git(["tag", tag])
        return "recreated"

    run_git(["tag", tag])
    return "created"


def parse_args(argv: Iterable[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Bump Android/iOS app version/build, optionally commit and tag. "
            "This command never pushes."
        ),
        epilog=(
            "Examples:\n"
            "  scripts/bump-version.py\n"
            "  scripts/bump-version.py --version 0.1.23\n"
            "  scripts/bump-version.py --version 0.1.23 --build 2 --retag\n"
            "  scripts/bump-version.py --version 0.1.23 --no-commit --no-tag"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--version",
        help=(
            "Semantic version X.Y.Z. If omitted, script reads remote tags and uses "
            "next patch version."
        ),
    )
    parser.add_argument(
        "--build",
        type=int,
        default=1,
        help=(
            "Build number (1..999), used as 3-digit suffix in versionCode/"
            "CURRENT_PROJECT_VERSION. Default: 1"
        ),
    )
    parser.add_argument(
        "--retag",
        action="store_true",
        help=(
            "Recreate existing local tag vX.Y.Z on current HEAD. "
            "Fails if remote tag exists."
        ),
    )
    parser.add_argument(
        "--no-commit",
        action="store_true",
        help="Do not create the 'Bump version' commit.",
    )
    parser.add_argument(
        "--no-tag",
        action="store_true",
        help="Do not create or recreate the git tag.",
    )

    args = parser.parse_args(list(argv))

    if args.build < 1 or args.build > 999:
        raise BumpError("Invalid --build. Expected integer in range 1..999")

    if args.version is not None:
        parse_version(args.version)
    if args.retag and args.no_tag:
        raise BumpError("--retag cannot be used together with --no-tag")

    return args


def main(argv: Iterable[str]) -> int:
    try:
        args = parse_args(argv)

        ensure_commit_preflight(args.no_commit)

        version = resolve_version(args.version)
        build_code = get_build_code(version, args.build)
        tag = f"v{version}"
        preflight_tag_policy(tag, args.no_tag, args.retag)

        update_android_project_file(version, build_code)
        update_ios_project_file(version, build_code)
        git_add_version_files()

        commit_result = commit_if_enabled(args.no_commit)
        tag_result = apply_tag(tag, args.no_tag, args.retag)

        print("Version bump completed")
        print(f"- version: {version}")
        print(f"- build: {args.build}")
        print(f"- build code: {build_code}")
        print(f"- commit: {commit_result}")
        print(f"- tag {tag}: {tag_result}")
        print(
            f"- files: {ANDROID_PROJECT_FILE.relative_to(REPO_ROOT)}, {IOS_PROJECT_FILE.relative_to(REPO_ROOT)}"
        )
        return 0
    except BumpError as exc:
        print(f"Error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
