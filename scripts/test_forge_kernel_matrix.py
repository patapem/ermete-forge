import unittest
import sys
import os
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))
import forge_kernel_matrix

class TestMatrixCLI(unittest.TestCase):
    def test_parse_args(self):
        sys.argv = ['forge-kernel-matrix.py', '--patch-dir', '/tmp/patches', '--kernel-src', '/tmp/linux']
        args = forge_kernel_matrix.parse_args()
        self.assertEqual(args.patch_dir, '/tmp/patches')
        self.assertEqual(args.kernel_src, '/tmp/linux')

if __name__ == '__main__':
    unittest.main()
