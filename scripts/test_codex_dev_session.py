#!/usr/bin/env python3
"""Regression tests for the codex-dev concurrent-session registry."""

from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("codex-dev-session.sh").resolve()


class SessionRegistryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.root = Path(self.tempdir.name)
        self.registry = self.root / "registry"
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        subprocess.run(
            ["git", "-C", str(self.root), "config", "user.email", "test@example.com"],
            check=True,
        )
        subprocess.run(
            ["git", "-C", str(self.root), "config", "user.name", "Test"],
            check=True,
        )
        (self.root / "tracked.txt").write_text("test\n")
        subprocess.run(["git", "-C", str(self.root), "add", "tracked.txt"], check=True)
        subprocess.run(
            ["git", "-C", str(self.root), "commit", "-qm", "test baseline"],
            check=True,
        )

    def tearDown(self) -> None:
        self.tempdir.cleanup()

    def run_session(self, *args: str) -> subprocess.CompletedProcess[str]:
        env = os.environ | {"CODEX_DEV_COORDINATION_DIR": str(self.registry)}
        return subprocess.run(
            [str(SCRIPT), *args],
            cwd=self.root,
            env=env,
            text=True,
            capture_output=True,
        )

    def test_claims_are_exclusive_and_integration_is_serialized(self) -> None:
        first = self.run_session("claim", "first", "tracked.txt")
        self.assertEqual(first.returncode, 0, first.stderr)

        conflict = self.run_session("claim", "second", "tracked.txt")
        self.assertNotEqual(conflict.returncode, 0)
        self.assertIn("path conflict", conflict.stderr)

        independent = self.run_session("claim", "second", "other.txt")
        self.assertEqual(independent.returncode, 0, independent.stderr)

        acquired = self.run_session("integration", "acquire", "integrator")
        self.assertEqual(acquired.returncode, 0, acquired.stderr)
        blocked = self.run_session("integration", "acquire", "other-integrator")
        self.assertNotEqual(blocked.returncode, 0)
        released = self.run_session("integration", "release", "integrator")
        self.assertEqual(released.returncode, 0, released.stderr)


if __name__ == "__main__":
    unittest.main()
