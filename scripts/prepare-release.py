#!/usr/bin/env python3
"""Build and verify private Linux release candidates without publishing them."""

import argparse
import hashlib
import io
import os
from pathlib import Path
import platform
import re
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parent.parent
TARGETS = {
    "x86_64-unknown-linux-gnu": ("x86_64", "Advanced Micro Devices X86-64"),
    "aarch64-unknown-linux-gnu": ("aarch64", "AArch64"),
}
MAX_GLIBC = (2, 35)
CONTENTS = ("gyro", "gyrognome", "gyrognome@.service", "INSTALL.md", "LICENSE",
            "THIRD_PARTY_NOTICES.md", "ProgressQuest-Desktop.txt",
            "ProgressQuest-Site.txt")


def run(*args, cwd=ROOT):
    result = subprocess.run(args, cwd=cwd, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(f"{args[0]} failed: {result.stderr.strip()}")
    return result.stdout.strip()


def check_elf(binary, target):
    header = run("readelf", "-h", str(binary))
    if f"Machine:                           {TARGETS[target][1]}" not in header:
        raise ValueError(f"{binary.name}: ELF architecture does not match {target}")
    versions = run("readelf", "--version-info", str(binary))
    required = [(int(major), int(minor)) for major, minor in
                re.findall(r"Name: GLIBC_(\d+)\.(\d+)", versions)]
    if not required or max(required) > MAX_GLIBC:
        raise ValueError(f"{binary.name}: unsupported or unknown GLIBC requirement")


def check_archive(archive, target, version):
    expected_prefix = f"gyrognome-{version}-{target}/"
    with tarfile.open(archive, "r:gz") as bundle:
        members = bundle.getmembers()
        names = {m.name for m in members}
        expected = {expected_prefix + name for name in CONTENTS}
        if names != expected or len(members) != len(expected):
            raise ValueError("archive contents differ from the release allowlist")
        for member in members:
            if not member.isfile() or member.issym() or member.islnk():
                raise ValueError("release archive contains a non-regular file")
            if member.name.endswith(("gyro", "gyrognome")) and not member.mode & 0o111:
                raise ValueError("release executable is not marked executable")
    digest_file = archive.with_name(archive.name + ".sha256")
    expected_digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    if digest_file.read_text() != f"{expected_digest}  {archive.name}\n":
        raise ValueError("archive checksum does not match")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--revision", required=True, help="full Git HEAD commit SHA")
    parser.add_argument("--output", type=Path, default=ROOT / "target" / "release-candidates")
    parser.add_argument("--allow-dirty", action="store_true", help="local trial only; not release-ready")
    args = parser.parse_args()
    if args.target not in TARGETS:
        parser.error(f"unsupported target: {args.target}")
    if platform.system() != "Linux" or platform.machine() != TARGETS[args.target][0]:
        parser.error(f"native Linux {TARGETS[args.target][0]} build host required")
    version = re.search(r'(?m)^version = "([^"]+)"$', (ROOT / "Cargo.toml").read_text())
    if version is None:
        parser.error("Cargo package version not found")
    version = version.group(1)
    if args.version != version or not re.fullmatch(r"\d+\.\d+\.\d+", version):
        parser.error("version must match the Cargo package version")
    revision = run("git", "rev-parse", "HEAD")
    if args.revision != revision:
        parser.error("revision must be the full Git HEAD commit SHA")
    if not args.allow_dirty and run("git", "status", "--porcelain", "--untracked-files=all"):
        parser.error("release candidate requires a clean worktree")
    if not args.allow_dirty and args.output.resolve().is_relative_to(ROOT):
        # Generated files live under target/ and cannot alter the source tree.
        if not args.output.resolve().is_relative_to(ROOT / "target"):
            parser.error("in-tree output must be under target/")

    run("cargo", "build", "--release", "--locked", "--bins", "--target", args.target)
    binary_dir = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / args.target / "release"
    for name in CONTENTS[:2]:
        check_elf(binary_dir / name, args.target)
        run(str(binary_dir / name), "--help")

    guide = (ROOT / "docs" / "linux-release-install.md").read_bytes()
    source_files = {
        "gyro": (binary_dir / "gyro").read_bytes(),
        "gyrognome": (binary_dir / "gyrognome").read_bytes(),
        "gyrognome@.service": (ROOT / "systemd/user/gyrognome@.service").read_bytes(),
        "INSTALL.md": guide,
        "LICENSE": (ROOT / "LICENSE").read_bytes(),
        "THIRD_PARTY_NOTICES.md": (ROOT / "THIRD_PARTY_NOTICES.md").read_bytes(),
        "ProgressQuest-Desktop.txt": (ROOT / "licenses/ProgressQuest-Desktop.txt").read_bytes(),
        "ProgressQuest-Site.txt": (ROOT / "licenses/ProgressQuest-Site.txt").read_bytes(),
    }
    args.output.mkdir(parents=True, exist_ok=True)
    archive = args.output / f"gyrognome-{version}-{args.target}.tar.gz"
    prefix = archive.name[:-7] + "/"
    with tarfile.open(archive, "w:gz") as bundle:
        for name, content in source_files.items():
            info = tarfile.TarInfo(prefix + name)
            info.size = len(content)
            info.mode = 0o755 if name in CONTENTS[:2] else 0o644
            bundle.addfile(info, io.BytesIO(content))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_name(archive.name + ".sha256").write_text(f"{digest}  {archive.name}\n")
    check_archive(archive, args.target, version)
    print(f"Private candidate: {archive} ({revision[:12]}, glibc <= 2.35)")
    if args.allow_dirty:
        print("WARNING: dirty local trial; NOT release-ready")


if __name__ == "__main__":
    main()
