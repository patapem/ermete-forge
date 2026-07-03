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

if __name__ == '__main__':
    unittest.main()
