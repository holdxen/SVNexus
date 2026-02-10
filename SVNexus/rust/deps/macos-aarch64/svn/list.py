import os
import sys

def list_files_relpath(target_dir):
    current_dir = os.getcwd()
    for root, dirs, files in os.walk(target_dir):
        for filename in files:
            full_path = os.path.join(root, filename)
            rel_path = os.path.relpath(full_path, current_dir)
            print(f'"{rel_path}",')

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("使用方法: python3 script.py <目标文件夹>")
        sys.exit(1)
    
    target_dir = sys.argv[1]
    
    if not os.path.isdir(target_dir):
        print(f"错误：'{target_dir}' 不是有效的目录")
        sys.exit(1)
    
    list_files_relpath(target_dir)