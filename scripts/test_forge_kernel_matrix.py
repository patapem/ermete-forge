import unittest
import sys
import os
from unittest.mock import patch, call
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))
import forge_kernel_matrix

class TestMatrixCLI(unittest.TestCase):
    def test_parse_args(self):
        args = forge_kernel_matrix.parse_args(['--patch-dir', '/tmp/patches', '--kernel-src', '/tmp/linux'])
        self.assertEqual(args.patch_dir, '/tmp/patches')
        self.assertEqual(args.kernel_src, '/tmp/linux')

    @patch('subprocess.run')
    def test_apply_patch_clean(self, mock_run):
        mock_run.return_value.returncode = 0
        mock_run.return_value.stdout = b"patching file fs/ext4/super.c\n"
        
        success, needs_val, files = forge_kernel_matrix.apply_patch("/tmp/test.patch", "/tmp/linux")
        self.assertTrue(success)
        self.assertFalse(needs_val)
        self.assertEqual(files, ["fs/ext4/super.c"])
        mock_run.assert_called_with(["patch", "-p1", "-i", "/tmp/test.patch"], cwd="/tmp/linux", capture_output=True)

    @patch('subprocess.run')
    def test_apply_patch_fuzz(self, mock_run):
        mock_run.return_value.returncode = 0
        mock_run.return_value.stdout = b"patching file fs/ext4/super.c\nHunk #1 succeeded at 100 with fuzz 2.\n"
        
        success, needs_val, files = forge_kernel_matrix.apply_patch("/tmp/test.patch", "/tmp/linux")
        self.assertTrue(success)
        self.assertTrue(needs_val)
        self.assertEqual(files, ["fs/ext4/super.c"])

    @patch('subprocess.run')
    def test_revert_patch(self, mock_run):
        forge_kernel_matrix.revert_patch("/tmp/test.patch", "/tmp/linux")
        mock_run.assert_called_with(["patch", "-R", "-p1", "-i", "/tmp/test.patch"], cwd="/tmp/linux", capture_output=True)

    @patch('subprocess.run')
    def test_validate_ast_native_success(self, mock_run):
        mock_run.return_value.returncode = 0
        is_valid = forge_kernel_matrix.validate_ast_native("fs/ext4/super.c", "/tmp/linux")
        self.assertTrue(is_valid)
        mock_run.assert_called_with(["make", "fs/ext4/super.o"], cwd="/tmp/linux", capture_output=True)

    @patch('subprocess.run')
    def test_validate_ast_native_failure(self, mock_run):
        mock_run.return_value.returncode = 2
        is_valid = forge_kernel_matrix.validate_ast_native("fs/ext4/super.c", "/tmp/linux")
        self.assertFalse(is_valid)
        mock_run.assert_called_with(["make", "fs/ext4/super.o"], cwd="/tmp/linux", capture_output=True)

    @patch('subprocess.run')
    def test_validate_ast_native_not_c(self, mock_run):
        is_valid = forge_kernel_matrix.validate_ast_native("fs/ext4/super.h", "/tmp/linux")
        self.assertTrue(is_valid)
        mock_run.assert_not_called()

    @patch('forge_kernel_matrix.revert_patch')
    @patch('forge_kernel_matrix.apply_patch')
    @patch('glob.glob')
    def test_process_patches_success(self, mock_glob, mock_apply, mock_revert):
        mock_glob.return_value = ["/tmp/patches/01.patch"]
        mock_apply.return_value = (True, False, ["fs/ext4/super.c"])
        
        forge_kernel_matrix.process_patches("/tmp/patches", "/tmp/linux")
        
        mock_apply.assert_called_once_with("/tmp/patches/01.patch", "/tmp/linux")
        mock_revert.assert_not_called()

    @patch('forge_kernel_matrix.revert_patch')
    @patch('forge_kernel_matrix.apply_patch')
    @patch('glob.glob')
    def test_process_patches_apply_failure(self, mock_glob, mock_apply, mock_revert):
        mock_glob.return_value = ["/tmp/patches/01.patch"]
        mock_apply.return_value = (False, False, [])
        
        forge_kernel_matrix.process_patches("/tmp/patches", "/tmp/linux")
        
        mock_apply.assert_called_once_with("/tmp/patches/01.patch", "/tmp/linux")
        mock_revert.assert_called_once_with("/tmp/patches/01.patch", "/tmp/linux")

    @patch('forge_kernel_matrix.validate_ast_native')
    @patch('forge_kernel_matrix.revert_patch')
    @patch('forge_kernel_matrix.apply_patch')
    @patch('glob.glob')
    def test_process_patches_fuzz_make_success(self, mock_glob, mock_apply, mock_revert, mock_validate):
        mock_glob.return_value = ["/tmp/patches/01.patch"]
        mock_apply.return_value = (True, True, ["fs/ext4/super.c"])
        mock_validate.return_value = True
        
        forge_kernel_matrix.process_patches("/tmp/patches", "/tmp/linux")
        
        mock_apply.assert_called_once_with("/tmp/patches/01.patch", "/tmp/linux")
        mock_validate.assert_called_once_with("fs/ext4/super.c", "/tmp/linux")
        mock_revert.assert_not_called()

    @patch('forge_kernel_matrix.validate_ast_native')
    @patch('forge_kernel_matrix.revert_patch')
    @patch('forge_kernel_matrix.apply_patch')
    @patch('glob.glob')
    def test_process_patches_fuzz_make_failure(self, mock_glob, mock_apply, mock_revert, mock_validate):
        mock_glob.return_value = ["/tmp/patches/01.patch"]
        mock_apply.return_value = (True, True, ["fs/ext4/super.c"])
        mock_validate.return_value = False
        
        forge_kernel_matrix.process_patches("/tmp/patches", "/tmp/linux")
        
        mock_apply.assert_called_once_with("/tmp/patches/01.patch", "/tmp/linux")
        mock_validate.assert_called_once_with("fs/ext4/super.c", "/tmp/linux")
        mock_revert.assert_called_once_with("/tmp/patches/01.patch", "/tmp/linux")

if __name__ == '__main__':
    unittest.main()
