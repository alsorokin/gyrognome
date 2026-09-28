import hashlib
import importlib.util
import io
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts/prepare-release.py"
spec = importlib.util.spec_from_file_location("prepare_release", SCRIPT)
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleaseCandidateTests(unittest.TestCase):
    def write_archive(self, directory, extra=None):
        target = "x86_64-unknown-linux-gnu"
        archive = directory / f"gyrognome-1.0.0-{target}.tar.gz"
        with tarfile.open(archive, "w:gz") as bundle:
            for name in (*release.CONTENTS, *(extra or ())):
                data = b"synthetic test content"
                info = tarfile.TarInfo(f"gyrognome-1.0.0-{target}/{name}")
                info.size = len(data)
                info.mode = 0o755 if name in release.CONTENTS[:2] else 0o644
                bundle.addfile(info, io.BytesIO(data))
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        archive.with_name(archive.name + ".sha256").write_text(f"{digest}  {archive.name}\n")
        return archive, target

    def test_allowlisted_bundle_verifies(self):
        with tempfile.TemporaryDirectory() as path:
            archive, target = self.write_archive(Path(path))
            release.check_archive(archive, target, "1.0.0")

    def test_rejects_extra_player_save(self):
        with tempfile.TemporaryDirectory() as path:
            archive, target = self.write_archive(Path(path), ["private.pqw"])
            with self.assertRaisesRegex(ValueError, "allowlist"):
                release.check_archive(archive, target, "1.0.0")

    def test_rejects_modified_archive(self):
        with tempfile.TemporaryDirectory() as path:
            archive, target = self.write_archive(Path(path))
            with archive.open("ab") as output:
                output.write(b"modified")
            with self.assertRaisesRegex(ValueError, "checksum"):
                release.check_archive(archive, target, "1.0.0")

    def test_unsupported_target_fails_before_build(self):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--target", "unknown", "--version",
             "1.0.0", "--revision", "0" * 40],
            text=True, capture_output=True,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unsupported target", result.stderr)


if __name__ == "__main__":
    unittest.main()
