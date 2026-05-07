#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
fix_rpath.py — 把 macOS 动态库 / 可执行文件中以绝对路径引用的依赖
改写为 @rpath/<dylib_name>，逐项交互确认。

使用：
    python3 fix_rpath.py <file1> [file2 ...]

工作流程：
    1. 用 `otool -D` 查询 install id（仅动态库有）。
       若 id 是绝对路径，询问是否改为 @rpath/<basename>。
    2. 用 `otool -L` 列出全部依赖。
       对每一条绝对路径依赖，询问是否改为 @rpath/<basename>。
    3. 全部修改通过 `install_name_tool` 完成，每次操作前都征求确认。
"""

import argparse
import os
import re
import subprocess
import sys


# ----------------------------- 终端着色 ----------------------------- #
class C:
    R = "\033[31m"     # red
    G = "\033[32m"     # green
    Y = "\033[33m"     # yellow
    B = "\033[34m"     # blue
    CY = "\033[36m"    # cyan
    BOLD = "\033[1m"
    DIM = "\033[2m"
    RST = "\033[0m"


# ----------------------------- 工具函数 ----------------------------- #
def run(cmd):
    """执行命令，返回 (stdout, stderr, returncode)。"""
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, check=False)
        return p.stdout, p.stderr, p.returncode
    except FileNotFoundError:
        print(f"{C.R}找不到命令：{cmd[0]}{C.RST}")
        sys.exit(1)


def check_tools():
    """确认 otool / install_name_tool 都在 PATH 中。"""
    for tool in ("otool", "install_name_tool"):
        _, _, rc = run(["which", tool])
        if rc != 0:
            print(f"{C.R}错误：找不到 {tool}。"
                  f"请在 macOS 上运行，并安装 Xcode 命令行工具"
                  f"（xcode-select --install）。{C.RST}")
            sys.exit(1)


def get_install_id(path):
    """
    返回动态库的 install id（LC_ID_DYLIB）。
    若文件是可执行文件 / 不是 Mach-O dylib，返回 None。
    """
    out, _, rc = run(["otool", "-D", path])
    if rc != 0:
        return None
    lines = out.strip().splitlines()
    # otool -D 的输出形如：
    #   <path>:
    #   <install_id>
    # 可执行文件只有第一行
    if len(lines) >= 2 and lines[1].strip():
        return lines[1].strip()
    return None


def get_deps(path, self_id=None):
    """
    返回依赖路径列表（仅路径，不含 compatibility/current version 信息）。
    若 self_id 给定且第一条依赖等于 self_id（动态库自身的 id 行），自动剔除。
    """
    out, err, rc = run(["otool", "-L", path])
    if rc != 0:
        print(f"{C.R}otool -L 执行失败：{err.strip()}{C.RST}")
        return []
    deps = []
    # 第一行是 "<file>:"，从第二行开始
    for line in out.strip().splitlines()[1:]:
        line = line.strip()
        if not line:
            continue
        # 形如：<path> (compatibility version x.x.x, current version y.y.y)
        m = re.match(r"^(.*?)\s+\(compatibility version .*\)\s*$", line)
        if m:
            deps.append(m.group(1).strip())
        else:
            # 兜底：取空格前的部分
            deps.append(line.split()[0])
    if self_id and deps and deps[0] == self_id:
        deps = deps[1:]
    return deps


def is_absolute_path(p):
    """是不是绝对路径（注意：@rpath/@loader_path/@executable_path 都不算）。"""
    return p.startswith("/")


def ask_yes_no(prompt, default_no=True):
    """询问用户 y/n。回车默认 No。"""
    suffix = " [y/N]: " if default_no else " [Y/n]: "
    while True:
        try:
            ans = input(f"{C.Y}{prompt}{C.RST}{suffix}").strip().lower()
        except (EOFError, KeyboardInterrupt):
            print()
            sys.exit(130)
        if ans == "":
            return not default_no if not default_no else False
        if ans in ("y", "yes"):
            return True
        if ans in ("n", "no"):
            return False
        print("  请输入 y 或 n。")


def warn_if_system_path(p):
    """对 /usr/lib、/System/Library 等系统库给出提醒（这些通常不该改）。"""
    sys_prefixes = ("/usr/lib/", "/System/Library/", "/Library/Apple/")
    if any(p.startswith(pref) for pref in sys_prefixes):
        print(f"  {C.R}⚠ 警告：这是系统库路径，通常不应改为 @rpath，"
              f"否则程序可能无法运行。{C.RST}")


def install_name_tool_id(path, new_id):
    out, err, rc = run(["install_name_tool", "-id", new_id, path])
    if rc != 0:
        print(f"{C.R}修改 id 失败：{err.strip() or out.strip()}{C.RST}")
        return False
    print(f"{C.G}  ✓ install id 已修改为：{new_id}{C.RST}")
    return True


def install_name_tool_change(path, old_dep, new_dep):
    out, err, rc = run(["install_name_tool", "-change", old_dep, new_dep, path])
    if rc != 0:
        print(f"{C.R}修改依赖失败：{err.strip() or out.strip()}{C.RST}")
        return False
    print(f"{C.G}  ✓ 依赖已修改：\n      {old_dep}\n   -> {new_dep}{C.RST}")
    return True


# ----------------------------- 主流程 ----------------------------- #
def process_file(path):
    if not os.path.isfile(path):
        print(f"{C.R}文件不存在：{path}{C.RST}")
        return

    abs_path = os.path.abspath(path)
    print(f"\n{C.BOLD}{C.B}{'=' * 70}{C.RST}")
    print(f"{C.BOLD}{C.B}处理文件：{abs_path}{C.RST}")
    print(f"{C.BOLD}{C.B}{'=' * 70}{C.RST}")

    # ---------- 1. install id ---------- #
    self_id = get_install_id(abs_path)
    if self_id is None:
        print(f"{C.CY}类型：可执行文件（无 install id）。{C.RST}")
    else:
        print(f"{C.CY}类型：动态库。{C.RST}")
        print(f"{C.CY}当前 install id：{C.BOLD}{self_id}{C.RST}")
        if is_absolute_path(self_id):
            new_id = f"@rpath/{os.path.basename(self_id)}"
            print(f"  建议改为：{C.G}{new_id}{C.RST}")
            warn_if_system_path(self_id)
            if ask_yes_no("是否修改 install id？"):
                if install_name_tool_id(abs_path, new_id):
                    self_id = new_id
            else:
                print(f"  {C.DIM}已跳过 install id。{C.RST}")
        else:
            print(f"  {C.G}install id 不是绝对路径，跳过。{C.RST}")

    # ---------- 2. 依赖列表 ---------- #
    deps = get_deps(abs_path, self_id=self_id)
    print(f"\n{C.BOLD}依赖共 {len(deps)} 个：{C.RST}")
    if not deps:
        print(f"  {C.DIM}（无）{C.RST}")
        return

    for i, d in enumerate(deps, 1):
        if is_absolute_path(d):
            tag = f"{C.Y}绝对路径{C.RST}"
        else:
            tag = f"{C.G}非绝对   {C.RST}"
        print(f"  {i:2d}. [{tag}] {d}")

    abs_deps = [d for d in deps if is_absolute_path(d)]
    if not abs_deps:
        print(f"\n{C.G}没有需要修改的绝对路径依赖。{C.RST}")
        return

    # ---------- 3. 逐个询问修改 ---------- #
    print(f"\n{C.BOLD}逐个询问 {len(abs_deps)} 条绝对路径依赖：{C.RST}")
    for d in abs_deps:
        new_dep = f"@rpath/{os.path.basename(d)}"
        print(f"\n{C.CY}依赖：{C.BOLD}{d}{C.RST}")
        print(f"  建议改为：{C.G}{new_dep}{C.RST}")
        warn_if_system_path(d)
        if ask_yes_no("是否修改此依赖？"):
            install_name_tool_change(abs_path, d, new_dep)
        else:
            print(f"  {C.DIM}已跳过。{C.RST}")

    print(f"\n{C.BOLD}{C.G}文件 {abs_path} 处理完成。{C.RST}")


def main():
    parser = argparse.ArgumentParser(
        description="把 macOS 动态库/可执行文件中以绝对路径引用的依赖"
                    "改写为 @rpath/<basename>，逐项交互确认。",
    )
    parser.add_argument("files", nargs="+",
                        help="要处理的 .dylib / .so / 可执行文件路径")
    args = parser.parse_args()

    if sys.platform != "darwin":
        print(f"{C.Y}注意：本脚本仅适用于 macOS。{C.RST}")

    check_tools()

    for f in args.files:
        process_file(f)

    print(f"\n{C.BOLD}全部完成。{C.RST}")
    print(f"{C.DIM}提示：install_name_tool 会使代码签名失效，"
          f"如有需要请重新 codesign（例如 `codesign --force --sign - <file>`）。{C.RST}")


if __name__ == "__main__":
    main()


