#!/usr/bin/env python3
import argparse
import sys
import os
import subprocess
import glob

def parse_args(args=None):
    parser = argparse.ArgumentParser(description="Universal Kernel Patch Matrix")
    parser.add_argument('--patch-dir', required=True, help="Directory containing patch files")
    parser.add_argument('--kernel-src', required=True, help="Kernel source directory")
    return parser.parse_args(args)

def apply_patch(patch_path, src_dir):
    result = subprocess.run(
        ["patch", "-p1", "-i", patch_path],
        cwd=src_dir,
        capture_output=True
    )
    stdout = result.stdout.decode('utf-8', errors='ignore')
    success = (result.returncode == 0)
    
    needs_validation = "fuzz" in stdout.lower() or "offset" in stdout.lower()
    files = []
    
    for line in stdout.splitlines():
        if line.startswith("patching file "):
            files.append(line.replace("patching file ", "").strip())
            
    return success, needs_validation, files

def revert_patch(patch_path, src_dir):
    subprocess.run(
        ["patch", "-R", "-p1", "-i", patch_path],
        cwd=src_dir,
        capture_output=True
    )

def validate_ast_native(file_path, src_dir):
    if not file_path.endswith('.c'):
        return True
    
    target = file_path[:-2] + ".o"
    result = subprocess.run(
        ["make", target],
        cwd=src_dir,
        capture_output=True
    )
    return result.returncode == 0

def process_patches(patch_dir, kernel_src):
    patches = sorted(glob.glob(os.path.join(patch_dir, "*.patch")))
    
    for patch_path in patches:
        abs_patch_path = os.path.abspath(patch_path)
        print(f"Processing {os.path.basename(abs_patch_path)}...")
        
        success, needs_validation, files = apply_patch(abs_patch_path, kernel_src)
        
        if not success:
            print(f"Patch failed to apply. Reverting...")
            revert_patch(abs_patch_path, kernel_src)
            print("Patch discarded.")
            continue
            
        if needs_validation:
            print(f"Fuzz/offset detected. Validating AST...")
            validation_failed = False
            for f in files:
                if f.endswith('.c'):
                    if not validate_ast_native(f, kernel_src):
                        validation_failed = True
                        break
                        
            if validation_failed:
                print(f"AST validation failed. Reverting...")
                revert_patch(abs_patch_path, kernel_src)
                print("Patch discarded.")
                continue
                
        print("Patch accepted.")

def main():
    args = parse_args()
    process_patches(args.patch_dir, args.kernel_src)

if __name__ == '__main__':
    main()
