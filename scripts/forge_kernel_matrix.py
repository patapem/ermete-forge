#!/usr/bin/env python3
import argparse
import sys
import os
import subprocess
import re

def parse_args(args=None):
    parser = argparse.ArgumentParser(description="Universal Kernel Patch Matrix")
    parser.add_argument('--patch-dir', required=True, help="Directory containing patch files")
    parser.add_argument('--kernel-src', required=True, help="Kernel source directory")
    return parser.parse_args(args)

def dry_run_patch(patch_path, src_dir):
    patch_path = os.path.abspath(patch_path)
    result = subprocess.run(
        ["patch", "-p1", "--dry-run", "-i", patch_path],
        cwd=src_dir,
        capture_output=True
    )
    stdout = result.stdout.decode('utf-8', errors='ignore')
    success = (result.returncode == 0)
    needs_ast = "fuzz" in stdout.lower() or "offset" in stdout.lower()
    
    files = []
    for line in stdout.splitlines():
        if line.startswith("patching file "):
            files.append(line.replace("patching file ", "").strip())
            
    return success, needs_ast, files

def extract_includes(file_path, src_dir):
    full_path = os.path.join(src_dir, file_path)
    include_dirs = []
    try:
        with open(full_path, 'r', encoding='utf-8') as f:
            content = f.read()
        headers = re.findall(r'#include\s*[<"]([^>"]+)[>"]', content)
        
        search_dirs = ["include", "arch/x86/include"]
        for sdir in search_dirs:
            for h in headers:
                if os.path.exists(os.path.join(src_dir, sdir, h)):
                    flag = f"-I{sdir}"
                    if flag not in include_dirs:
                        include_dirs.append(flag)
    except FileNotFoundError:
        pass
    return include_dirs

def validate_ast(file_path, src_dir):
    # Ponytail: Basic header extraction for clang
    includes = extract_includes(file_path, src_dir)
    clang_cmd = ["clang", "-fsyntax-only"] + includes + [file_path]
    try:
        result = subprocess.run(clang_cmd, cwd=src_dir, capture_output=True)
        return result.returncode == 0
    except FileNotFoundError:
        print("Error: clang not found", file=sys.stderr)
        return False

if __name__ == '__main__':
    args = parse_args()
