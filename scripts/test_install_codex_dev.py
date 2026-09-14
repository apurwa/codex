"""Exercise the local installer without touching the real user's installation."""

import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


INSTALLER = Path(__file__).with_name("install-codex-dev.sh").resolve()


class InstallCodexDevTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="codex-installer-test-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.prefix = self.root / "installation"
        self.binary = self.root / "codex"
        self.binary.write_text("#!/bin/sh\necho 'codex-cli test-build'\n")
        self.binary.chmod(0o755)
        self.git("init", "-q", "--initial-branch=main")
        self.git("config", "user.name", "Installer Test")
        self.git("config", "user.email", "installer@example.invalid")
        (self.repo / "source").write_text("verified source\n")
        self.git("add", "source")
        self.git("commit", "-qm", "Test integration")
        integration = self.repo / ".worktrees/integration"
        self.git("worktree", "add", "-qb", "integration/codex-dev", str(integration))
        self.repo = integration

    def git(self, *arguments):
        return subprocess.run(
            ["git", *arguments],
            cwd=self.repo,
            check=True,
            text=True,
            capture_output=True,
        ).stdout.strip()

    def install(self):
        return subprocess.run(
            ["sh", str(INSTALLER), str(self.binary)],
            cwd=self.repo,
            env={**os.environ, "CODEX_DEV_INSTALL_ROOT": str(self.prefix)},
            text=True,
            capture_output=True,
        )

    def test_installs_exact_binary_records_revision_and_preserves_previous(self):
        result = self.install()
        self.assertEqual(result.returncode, 0, result.stderr)
        target = self.prefix / "libexec/codex-dev-bin"
        original = target.read_bytes()
        self.assertEqual(original, self.binary.read_bytes())
        receipt = (self.prefix / "libexec/codex-dev-install.txt").read_text()
        self.assertIn(self.git("rev-parse", "HEAD"), receipt)
        self.assertIn(hashlib.sha256(original).hexdigest(), receipt)
        self.binary.write_text("#!/bin/sh\necho 'codex-cli next-test-build'\n")
        self.assertEqual(self.install().returncode, 0)
        self.assertEqual(
            (target.with_name("codex-dev-bin.previous")).read_bytes(), original
        )
        self.assertEqual(target.read_bytes(), self.binary.read_bytes())
        self.assertFalse(target.with_name("codex-dev-install.lock").exists())

    def test_feature_branch_cannot_install(self):
        self.git("switch", "-qc", "feat/example")
        self.assertNotEqual(self.install().returncode, 0)
        self.assertFalse(self.prefix.exists())

    def test_integration_branch_outside_canonical_worktree_cannot_install(self):
        moved = self.root / "misplaced-integration"
        self.git("worktree", "move", str(self.repo), str(moved))
        self.repo = moved
        self.assertNotEqual(self.install().returncode, 0)
        self.assertFalse(self.prefix.exists())

    def test_dirty_integration_cannot_install(self):
        (self.repo / "source").write_text("untested edits\n")
        self.assertNotEqual(self.install().returncode, 0)
        self.assertFalse(self.prefix.exists())

    def test_active_install_lock_is_preserved(self):
        lock = self.prefix / "libexec/codex-dev-install.lock"
        lock.mkdir(parents=True)
        self.assertNotEqual(self.install().returncode, 0)
        self.assertTrue(lock.is_dir())
        self.assertFalse(lock.with_name("codex-dev-bin").exists())

    def test_untracked_source_cannot_be_omitted_from_install_revision(self):
        (self.repo / "new-source").write_text("uncommitted source\n")
        self.assertNotEqual(self.install().returncode, 0)
        self.assertFalse(self.prefix.exists())

    def test_non_codex_executable_is_rejected(self):
        self.binary.write_text("#!/bin/sh\necho 'wrong program'\n")
        self.assertNotEqual(self.install().returncode, 0)
        self.assertFalse(self.prefix.exists())


if __name__ == "__main__":
    unittest.main()
