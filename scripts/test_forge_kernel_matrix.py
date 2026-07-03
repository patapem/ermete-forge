import unittest
import sys
import os
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))
import forge_kernel_matrix

class TestMatrixCLI(unittest.TestCase):
    def test_parse_args(self):
        args = forge_kernel_matrix.parse_args(['--patch-dir', '/tmp/patches', '--kernel-src', '/tmp/linux'])
        self.assertEqual(args.patch_dir, '/tmp/patches')
        self.assertEqual(args.kernel_src, '/tmp/linux')

    def test_dry_run_clean(self):
        # We simulate subprocess.run using unittest.mock
        from unittest.mock import patch
        with patch('subprocess.run') as mock_run:
            mock_run.return_value.returncode = 0
            mock_run.return_value.stdout = b"patching file fs/ext4/super.c\n"
            mock_run.return_value.stderr = b""
            success, needs_ast, files = forge_kernel_matrix.dry_run_patch("test.patch", "/tmp/linux")
            self.assertTrue(success)
            self.assertFalse(needs_ast)
            self.assertEqual(files, ["fs/ext4/super.c"])
            
    def test_dry_run_fuzz(self):
        from unittest.mock import patch
        with patch('subprocess.run') as mock_run:
            mock_run.return_value.returncode = 0
            mock_run.return_value.stdout = b"patching file fs/ext4/super.c\nHunk #1 succeeded at 100 with fuzz 2.\n"
            mock_run.return_value.stderr = b""
            success, needs_ast, files = forge_kernel_matrix.dry_run_patch("test.patch", "/tmp/linux")
            self.assertTrue(success)
            self.assertTrue(needs_ast)
            self.assertEqual(files, ["fs/ext4/super.c"])

    def test_validate_ast(self):
        from unittest.mock import patch, mock_open
        with patch('subprocess.run') as mock_run:
            mock_run.return_value.returncode = 0
            
            mock_content = '#include <linux/fs.h>\n#include <asm/page.h>'
            with patch('builtins.open', mock_open(read_data=mock_content)):
                with patch('os.path.exists') as mock_exists:
                    def exists_side_effect(path):
                        if path.endswith('include/linux/fs.h'): return True
                        if path.endswith('arch/x86/include/asm/page.h'): return True
                        return False
                    mock_exists.side_effect = exists_side_effect
                    
                    is_valid = forge_kernel_matrix.validate_ast("fs/ext4/super.c", "/tmp/linux")
                    self.assertTrue(is_valid)
                    mock_run.assert_called_with(
                        ["clang", "-fsyntax-only", "-Iinclude", "-Iarch/x86/include", "fs/ext4/super.c"],
                        cwd="/tmp/linux",
                        capture_output=True
                    )

    def test_validate_ast_clang_missing(self):
        from unittest.mock import patch, mock_open
        with patch('subprocess.run') as mock_run:
            mock_run.side_effect = FileNotFoundError()
            
            mock_content = '#include <linux/fs.h>\n'
            with patch('builtins.open', mock_open(read_data=mock_content)):
                with patch('os.path.exists') as mock_exists:
                    mock_exists.return_value = True
                    is_valid = forge_kernel_matrix.validate_ast("fs/ext4/super.c", "/tmp/linux")
                    self.assertFalse(is_valid)

if __name__ == '__main__':
    unittest.main()
