#!/bin/sh
# Install a tested integration binary; never change the official codex command.
set -eu

repo=$(git rev-parse --show-toplevel)
branch=$(git -C "$repo" branch --show-current)
if [ "$branch" != integration/codex-dev ]; then
  echo 'Install only from the integration/codex-dev worktree.' >&2
  exit 1
fi
if ! git -C "$repo" diff --quiet || ! git -C "$repo" diff --cached --quiet; then
  echo 'Commit the integration changes before installing.' >&2
  exit 1
fi
binary=${1:?Usage: scripts/install-codex-dev.sh /absolute/path/to/tested/codex}
case "$binary" in /*) ;; *) echo 'Use an absolute binary path.' >&2; exit 1 ;; esac
test -f "$binary" && test -x "$binary"
"$binary" --version

install_root=${CODEX_DEV_INSTALL_ROOT:-"$HOME/.local"}
libexec="$install_root/libexec"
mkdir -p "$libexec"
lock="$libexec/codex-dev-install.lock"
if ! mkdir "$lock" 2>/dev/null; then
  echo "Another installer owns $lock; do not remove an active lock." >&2
  exit 1
fi
trap 'rmdir "$lock"' EXIT
stage=$(mktemp "$libexec/codex-dev-bin.stage.XXXXXX")
install -m 755 "$binary" "$stage"
if [ -f "$libexec/codex-dev-bin" ]; then
  cp -p "$libexec/codex-dev-bin" "$libexec/codex-dev-bin.previous"
fi
mv -f "$stage" "$libexec/codex-dev-bin"
{
  git -C "$repo" rev-parse HEAD
  shasum -a 256 "$libexec/codex-dev-bin"
  date -u '+Installed %Y-%m-%dT%H:%M:%SZ'
} > "$libexec/codex-dev-install.txt"
echo "Installed $branch. Restart codex-dev agents to use it."
