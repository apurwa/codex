#!/bin/sh
# Verify that the installed codex-dev binary is the receipt-pinned build.
set -eu

repo=$(git rev-parse --show-toplevel)
branch=$(git -C "$repo" branch --show-current)
if [ "$branch" != integration/codex-dev ]; then
  echo "Expected integration/codex-dev, found $branch." >&2
  exit 1
fi

install_root=${CODEX_DEV_INSTALL_ROOT:-"$HOME/.local"}
libexec="$install_root/libexec"
binary=${CODEX_DEV_BINARY:-"$libexec/codex-dev-bin"}
receipt=${CODEX_DEV_RECEIPT:-"$libexec/codex-dev-install.txt"}

test -x "$binary" || {
  echo "Installed binary is missing or not executable: $binary" >&2
  exit 1
}
test -f "$receipt" || {
  echo "Install receipt is missing: $receipt" >&2
  exit 1
}

installed_commit=$(sed -n 's/^integration_commit=//p' "$receipt")
expected_hash=$(sed -n 's/^binary_sha256=//p' "$receipt")
if [ -z "$installed_commit" ] || [ -z "$expected_hash" ]; then
  # Accept receipts written by the pre-provenance installer.
  installed_commit=$(sed -n '1p' "$receipt")
  expected_hash=$(sed -n '2s/[[:space:]].*$//p' "$receipt")
fi
if ! git -C "$repo" rev-parse --verify "$installed_commit^{commit}" >/dev/null 2>&1; then
  echo "Receipt does not identify a Git commit: $installed_commit" >&2
  exit 1
fi
if [ "${#expected_hash}" -ne 64 ] \
  || [ "$(printf '%s' "$expected_hash" | tr -cd '0123456789abcdefABCDEF')" != "$expected_hash" ]; then
  echo "Receipt does not contain a binary SHA-256: $expected_hash" >&2
  exit 1
fi

actual_hash=$(shasum -a 256 "$binary" | sed 's/[[:space:]].*$//')
if [ "$actual_hash" != "$expected_hash" ]; then
  echo "Installed binary hash mismatch." >&2
  echo "  expected: $expected_hash" >&2
  echo "  actual:   $actual_hash" >&2
  exit 1
fi

if ! git -C "$repo" merge-base --is-ancestor "$installed_commit" HEAD; then
  echo "Receipt commit $installed_commit is not an ancestor of integration HEAD." >&2
  exit 1
fi

echo "Verified codex-dev installation."
echo "  commit: $installed_commit"
echo "  sha256: $actual_hash"
echo "  binary: $binary"
