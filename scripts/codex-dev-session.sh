#!/bin/sh
# Coordinate concurrent codex-dev sessions without editing the shared integration worktree.
set -eu

repo=$(git rev-parse --show-toplevel)
common_dir=$(git rev-parse --path-format=absolute --git-common-dir)
coordination_dir=${CODEX_DEV_COORDINATION_DIR:-"$common_dir/codex-dev-coordination"}
claims_dir="$coordination_dir/claims"
handoffs_dir="$coordination_dir/handoffs"
integration_lock="$coordination_dir/integration.lock"

die() {
  echo "codex-dev session: $*" >&2
  exit 1
}

usage() {
  cat >&2 <<'EOF'
Usage:
  scripts/codex-dev-session.sh claim <session-id> <path> [path ...]
  scripts/codex-dev-session.sh finish <session-id> <commit> <validation> [notes ...]
  scripts/codex-dev-session.sh release <session-id>
  scripts/codex-dev-session.sh status
  scripts/codex-dev-session.sh integration acquire <session-id>
  scripts/codex-dev-session.sh integration release <session-id>

Claims are stored under the shared Git directory, not in the worktree.
Paths are repository-relative and are exclusive while a claim is active.
EOF
  exit 2
}

valid_id() {
  case "$1" in
    ''|*[!A-Za-z0-9._-]*) return 1 ;;
    *) return 0 ;;
  esac
}

normalize_path() {
  path=$1
  case "$path" in
    /*) die "claim paths must be repository-relative: $path" ;;
    .) path='' ;;
    ./*) path=${path#./} ;;
  esac
  [ -n "$path" ] || die 'claim paths cannot be empty'
  printf '%s\n' "$path"
}

path_overlaps() {
  requested=$1
  claimed=$2
  if [ "$requested" = "$claimed" ]; then
    return 0
  fi
  case "$requested" in
    "$claimed"/*) return 0 ;;
  esac
  case "$claimed" in
    "$requested"/*) return 0 ;;
  esac
  return 1
}

with_registry_lock() {
  lock="$coordination_dir/.registry.lock"
  mkdir "$lock" 2>/dev/null || die "coordination registry is busy: $lock"
  trap 'rmdir "$lock" 2>/dev/null || true' EXIT HUP INT TERM
  "$@"
  trap - EXIT HUP INT TERM
  rmdir "$lock"
}

claim_impl() {
  session_id=$1
  shift
  valid_id "$session_id" || die "invalid session id: $session_id"
  [ "$#" -gt 0 ] || die 'claim requires at least one path'
  claim_file="$claims_dir/$session_id.claim"
  [ ! -e "$claim_file" ] || die "session already has an active claim: $session_id"

  requested_paths=''
  for path in "$@"; do
    normalized=$(normalize_path "$path")
    requested_paths="${requested_paths}${normalized}
"
  done

  for existing in "$claims_dir"/*.claim; do
    [ -e "$existing" ] || continue
    [ "$existing" = "$claim_file" ] && continue
    status=$(sed -n 's/^status=//p' "$existing")
    [ "$status" = active ] || continue
    while IFS= read -r claimed_path; do
      [ -n "$claimed_path" ] || continue
      while IFS= read -r requested_path; do
        [ -n "$requested_path" ] || continue
        if path_overlaps "$requested_path" "$claimed_path"; then
          owner=$(sed -n 's/^session_id=//p' "$existing")
          die "path conflict: $requested_path is claimed by $owner"
        fi
      done <<EOF_PATHS
$requested_paths
EOF_PATHS
    done <<EOF_CLAIM
$(sed -n 's/^path=//p' "$existing")
EOF_CLAIM
  done

  tmp=$(mktemp "$coordination_dir/.claim.XXXXXX")
  {
    printf 'session_id=%s\n' "$session_id"
    printf 'status=active\n'
    printf 'branch=%s\n' "$(git branch --show-current)"
    printf 'worktree=%s\n' "$repo"
    printf 'base_commit=%s\n' "$(git rev-parse HEAD)"
    printf 'created_at=%s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    printf '%s' "$requested_paths" | while IFS= read -r path; do
      [ -n "$path" ] && printf 'path=%s\n' "$path"
    done
  } >"$tmp"
  mv "$tmp" "$claim_file"
  echo "Claimed $session_id: $(sed -n 's/^path=//p' "$claim_file" | tr '\n' ' ')"
}

finish_impl() {
  session_id=$1
  commit=$2
  validation=$3
  shift 3
  valid_id "$session_id" || die "invalid session id: $session_id"
  claim_file="$claims_dir/$session_id.claim"
  [ -f "$claim_file" ] || die "no active claim for session: $session_id"
  git rev-parse --verify "$commit^{commit}" >/dev/null 2>&1 \
    || die "unknown commit: $commit"

  note=$*
  handoff_file="$handoffs_dir/$session_id.md"
  {
    printf '# Codex-dev session handoff: %s\n\n' "$session_id"
    printf -- '- Branch: `%s`\n' "$(sed -n 's/^branch=//p' "$claim_file")"
    printf -- '- Worktree: `%s`\n' "$(sed -n 's/^worktree=//p' "$claim_file")"
    printf -- '- Base commit: `%s`\n' "$(sed -n 's/^base_commit=//p' "$claim_file")"
    printf -- '- Commit: `%s`\n' "$commit"
    printf -- '- Validation: %s\n' "$validation"
    printf -- '- Finished: %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    if [ -n "$note" ]; then
      printf '\n%s\n' "$note"
    fi
    printf '\n## Claimed paths\n\n'
    sed -n 's/^path=- /- /p' "$claim_file"
    sed -n 's/^path=/\- `/p' "$claim_file" | sed 's/$/`/'
  } >"$handoff_file"
  mv "$claim_file" "$coordination_dir/archive-$session_id-$(date -u '+%Y%m%dT%H%M%SZ').claim"
  echo "Wrote handoff $handoff_file"
}

release_impl() {
  session_id=$1
  valid_id "$session_id" || die "invalid session id: $session_id"
  claim_file="$claims_dir/$session_id.claim"
  [ -f "$claim_file" ] || die "no active claim for session: $session_id"
  rm "$claim_file"
  echo "Released $session_id"
}

status_impl() {
  printf 'Coordination registry: %s\n' "$coordination_dir"
  found=false
  for claim in "$claims_dir"/*.claim; do
    [ -e "$claim" ] || continue
    found=true
    printf '\n%s\n' "$(sed -n 's/^session_id=//p' "$claim")"
    sed -n -e 's/^status=/  status: /' -e 's/^branch=/  branch: /' \
      -e 's/^worktree=/  worktree: /' -e 's/^path=/  path: /' "$claim"
  done
  if [ "$found" = false ]; then
    echo 'No active feature claims.'
  fi
  if [ -f "$integration_lock/owner" ]; then
    printf '\nIntegration owner: %s\n' "$(cat "$integration_lock/owner")"
  else
    echo 'Integration owner: none'
  fi
}

integration_impl() {
  action=${1:-}
  session_id=${2:-}
  valid_id "$session_id" || die "invalid session id: $session_id"
  case "$action" in
    acquire)
      mkdir "$integration_lock" 2>/dev/null \
        || die "integration is already owned by $(cat "$integration_lock/owner" 2>/dev/null || echo another session)"
      printf '%s\n' "$session_id" >"$integration_lock/owner"
      printf '%s\n' "$(git rev-parse HEAD)" >"$integration_lock/base_commit"
      echo "Integration ownership acquired by $session_id"
      ;;
    release)
      [ -f "$integration_lock/owner" ] || die 'integration is not currently owned'
      owner=$(cat "$integration_lock/owner")
      [ "$owner" = "$session_id" ] || die "integration is owned by $owner"
      rm -rf "$integration_lock"
      echo "Integration ownership released by $session_id"
      ;;
    *) usage ;;
  esac
}

[ "$#" -gt 0 ] || usage
mkdir -p "$claims_dir" "$handoffs_dir"
command=$1
shift
case "$command" in
  claim) [ "$#" -ge 2 ] || usage; with_registry_lock claim_impl "$@" ;;
  finish) [ "$#" -ge 3 ] || usage; with_registry_lock finish_impl "$@" ;;
  release) [ "$#" -eq 1 ] || usage; with_registry_lock release_impl "$@" ;;
  status) [ "$#" -eq 0 ] || usage; status_impl ;;
  integration) with_registry_lock integration_impl "$@" ;;
  *) usage ;;
esac
