#!/usr/bin/env python3
import os
import stat
import subprocess
from pathlib import Path


TARGET_PREFIX = "svn"
NEW_RPATH = "@loader_path/../lib"


def run(cmd, check=False):
    print("+", " ".join(cmd))
    return subprocess.run(
        cmd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=check,
    )


def is_executable(path: Path) -> bool:
    try:
        mode = path.stat().st_mode
        return bool(mode & (stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH))
    except OSError:
        return False


def is_macho_file(path: Path) -> bool:
    result = run(["file", str(path)], check=False)
    output = result.stdout + result.stderr
    return "Mach-O" in output


def is_macho_executable(path: Path) -> bool:
    result = run(["file", str(path)], check=False)
    output = result.stdout + result.stderr

    return (
        "Mach-O" in output
        and (
            "executable" in output
            or "universal binary" in output
        )
    )


def get_rpaths(path: Path) -> list[str]:
    result = run(["otool", "-l", str(path)], check=False)

    if result.returncode != 0:
        print(f"跳过，otool 失败: {path}")
        print(result.stderr.strip())
        return []

    rpaths = []
    lines = result.stdout.splitlines()

    for i, line in enumerate(lines):
        if line.strip() == "cmd LC_RPATH":
            for j in range(i, min(i + 8, len(lines))):
                item = lines[j].strip()
                if item.startswith("path "):
                    # 形如：path @loader_path/../lib (offset 12)
                    rpath = item.split(" ", 2)[1]
                    rpaths.append(rpath)

    return rpaths


def delete_rpath(path: Path, rpath: str):
    result = run(
        ["install_name_tool", "-delete_rpath", rpath, str(path)],
        check=False,
    )

    if result.returncode != 0:
        print(f"删除 rpath 失败，可忽略: {path} -> {rpath}")
        print(result.stderr.strip())


def add_rpath(path: Path, rpath: str):
    result = run(
        ["install_name_tool", "-add_rpath", rpath, str(path)],
        check=False,
    )

    if result.returncode != 0:
        print(f"添加 rpath 失败: {path} -> {rpath}")
        print(result.stderr.strip())


def fix_one_file(path: Path, dry_run: bool = False):
    print(f"\n处理: {path}")

    old_rpaths = get_rpaths(path)

    if dry_run:
        print("当前 rpath:")
        if old_rpaths:
            for r in old_rpaths:
                print("  ", r)
        else:
            print("  无")
        print(f"将设置为: {NEW_RPATH}")
        return

    for rpath in old_rpaths:
        delete_rpath(path, rpath)

    add_rpath(path, NEW_RPATH)

    new_rpaths = get_rpaths(path)
    print("新的 rpath:")
    if new_rpaths:
        for r in new_rpaths:
            print("  ", r)
    else:
        print("  无")


def main():
    root = Path.cwd()
    dry_run = "--dry-run" in os.sys.argv

    print(f"扫描目录: {root}")
    print(f"目标文件名前缀: {TARGET_PREFIX}")
    print(f"目标 rpath: {NEW_RPATH}")
    print(f"dry-run: {dry_run}")

    candidates = []

    for path in root.rglob("*"):
        if not path.is_file():
            continue

        if not path.name.startswith(TARGET_PREFIX):
            continue

        if not is_executable(path):
            continue

        if not is_macho_executable(path):
            continue

        candidates.append(path)

    if not candidates:
        print("没有找到文件名以 svn 开头的 macOS Mach-O 可执行文件")
        return

    print(f"找到 {len(candidates)} 个目标文件")

    for path in candidates:
        fix_one_file(path, dry_run=dry_run)


if __name__ == "__main__":
    main()

