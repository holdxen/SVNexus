#!/usr/bin/env python3
import os
import subprocess
from pathlib import Path


TARGET_SUFFIXES = {".dylib", ".so", ".bundle"}


def run(cmd, check=True):
    print("+", " ".join(cmd))
    return subprocess.run(
        cmd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=check,
    )


def is_macho_file(path: Path) -> bool:
    try:
        result = run(["file", str(path)], check=False)
        output = result.stdout + result.stderr
        return "Mach-O" in output
    except Exception:
        return False


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
                    # 形如：path @loader_path (offset 12)
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
        # 如果已经存在，install_name_tool 会报错；这里不当成致命错误
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
        print("将设置为: @loader_path")
        return

    for rpath in old_rpaths:
        delete_rpath(path, rpath)

    add_rpath(path, "@loader_path")

    new_rpaths = get_rpaths(path)
    print("新的 rpath:")
    for r in new_rpaths:
        print("  ", r)


def main():
    root = Path.cwd()

    dry_run = "--dry-run" in os.sys.argv

    print(f"扫描目录: {root}")
    print(f"dry-run: {dry_run}")

    candidates = []

    for path in root.rglob("*"):
        if not path.is_file():
            continue

        if path.suffix not in TARGET_SUFFIXES:
            continue

        if not is_macho_file(path):
            continue

        candidates.append(path)

    if not candidates:
        print("没有找到 macOS Mach-O 动态库文件")
        return

    print(f"找到 {len(candidates)} 个动态库文件")

    for path in candidates:
        fix_one_file(path, dry_run=dry_run)


if __name__ == "__main__":
    main()

