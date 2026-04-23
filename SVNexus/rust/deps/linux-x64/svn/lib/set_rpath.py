#!/usr/bin/env python3
"""
搜索当前目录下所有 .so 文件，并使用 patchelf 设置 rpath 为 $ORIGIN
用法: python3 fix_rpath.py
"""

import os
import subprocess
import sys


def find_so_files(root_dir="."):
    """递归查找所有 .so 结尾的文件"""
    so_files = []
    for dirpath, _, filenames in os.walk(root_dir):
        for fname in filenames:
            if fname.endswith(".so") or ".so." in fname:
                so_files.append(os.path.join(dirpath, fname))
    return so_files


def set_rpath(filepath):
    """使用 patchelf 设置 rpath 为 $ORIGIN"""
    try:
        # patchelf --set-rpath '$ORIGIN' <file>
        result = subprocess.run(
            ["patchelf", "--set-rpath", "$ORIGIN", filepath],
            capture_output=True,
            text=True,
        )
        if result.returncode == 0:
            print(f"  [OK]  {filepath}")
        else:
            print(f"  [FAIL] {filepath}  ->  {result.stderr.strip()}")
        return result.returncode == 0
    except FileNotFoundError:
        print("错误: 未找到 patchelf 命令，请先安装 patchelf")
        sys.exit(1)


def main():
    print("=== patchelf rpath 修复工具 ===\n")
    print(f"搜索目录: {os.path.abspath('.')}\n")

    so_files = find_so_files(".")

    if not so_files:
        print("未找到任何 .so 文件。")
        return

    print(f"找到 {len(so_files)} 个 .so 文件:\n")

    success, fail = 0, 0
    for f in so_files:
        if set_rpath(f):
            success += 1
        else:
            fail += 1

    print(f"\n完成: {success} 成功, {fail} 失败, 共 {len(so_files)} 个文件")


if __name__ == "__main__":
    main()

