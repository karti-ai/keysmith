#!/usr/bin/env bash
# Guarded static release deployment for https://keychron.karti.ai/.
# Builds an exact clean Git commit, uploads an immutable release, atomically
# switches the current symlink, validates/reloads Caddy, and rolls back on a
# failed public smoke test. It never touches DNS or the private Keysmith API.
set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO=$(cd -- "$SCRIPT_DIR/../.." && pwd)
SITE_DIR="$REPO/apps/site"
DIST_DIR="$SITE_DIR/dist/client"
PUBLIC_URL=https://keychron.karti.ai/
DEPLOY_HOST=${KEYSMITH_DEPLOY_HOST:-cloud-2}
REMOTE_ROOT=/var/www/keychron.karti.ai
RELEASES="$REMOTE_ROOT/releases"
CONFIGS="$REMOTE_ROOT/configs"
SHA_PATTERN='^[0-9a-f]{40}$'
deploy_lock_held=0
deploy_lock_token="$(date +%s)-$$"

usage() {
  cat >&2 <<'USAGE'
usage:
  deploy/public/deploy.sh deploy <exact-40-character-source-sha>
  deploy/public/deploy.sh rollback <existing-40-character-release-sha>
  deploy/public/deploy.sh status

Environment:
  KEYSMITH_DEPLOY_HOST   SSH alias or user@host (default: cloud-2)
USAGE
  exit 2
}

if [[ ! "$DEPLOY_HOST" =~ ^[A-Za-z0-9._@:-]+$ || "$DEPLOY_HOST" == -* ]]; then
  echo "deploy: unsafe KEYSMITH_DEPLOY_HOST" >&2
  exit 2
fi

command=${1:-}
source_sha=${2:-}
case "$command" in
  deploy|rollback)
    [[ "$source_sha" =~ $SHA_PATTERN ]] || usage
    ;;
  status)
    [[ $# -eq 1 ]] || usage
    ;;
  *) usage ;;
esac

remote_bootstrap() {
  ssh -o BatchMode=yes "$DEPLOY_HOST" bash -s -- "$REMOTE_ROOT" <<'REMOTE'
set -euo pipefail
root=$1
[[ "$root" == /var/www/keychron.karti.ai ]] || exit 2
if [[ -e "$root" && ( ! -d "$root" || -L "$root" ) ]]; then
  echo "deploy: remote root has an unexpected type: $root" >&2
  exit 1
fi
for child in "$root/releases" "$root/configs"; do
  if [[ -e "$child" && ( ! -d "$child" || -L "$child" ) ]]; then
    echo "deploy: remote deployment directory has an unexpected type: $child" >&2
    exit 1
  fi
done
owner=$(id -un)
group=$(id -gn)
sudo install -d -o "$owner" -g "$group" -m 0755 \
  "$root" "$root/releases" "$root/configs"
REMOTE
}

acquire_deploy_lock() {
  ssh -o BatchMode=yes "$DEPLOY_HOST" bash -s -- "$REMOTE_ROOT" "$deploy_lock_token" <<'REMOTE'
set -euo pipefail
root=$1
token=$2
[[ "$root" == /var/www/keychron.karti.ai && "$token" =~ ^[0-9]+-[0-9]+$ ]] || exit 2
[[ -d "$root" && ! -L "$root" ]] || { echo "deploy: remote root does not exist" >&2; exit 1; }
lock="$root/.deploy-lock"
if ! mkdir -- "$lock"; then
  echo "deploy: another deployment holds $lock (inspect before removing a stale lock)" >&2
  exit 1
fi
printf '%s\n' "$token" > "$lock/owner"
REMOTE
  deploy_lock_held=1
}

release_deploy_lock() {
  (( deploy_lock_held == 1 )) || return 0
  if ! ssh -o BatchMode=yes "$DEPLOY_HOST" bash -s -- "$REMOTE_ROOT" "$deploy_lock_token" <<'REMOTE'
set -euo pipefail
root=$1
token=$2
[[ "$root" == /var/www/keychron.karti.ai && "$token" =~ ^[0-9]+-[0-9]+$ ]] || exit 2
lock="$root/.deploy-lock"
[[ -d "$lock" && ! -L "$lock" && -f "$lock/owner" && ! -L "$lock/owner" ]] || exit 1
[[ "$(cat "$lock/owner")" == "$token" ]] || exit 1
rm -- "$lock/owner"
rmdir -- "$lock"
REMOTE
  then
    echo "deploy: warning: could not release the owned remote deployment lock" >&2
    return 1
  fi
  deploy_lock_held=0
}

cleanup() {
  release_deploy_lock || true
}
trap cleanup EXIT

remote_verify_release() {
  local sha=$1
  ssh -o BatchMode=yes "$DEPLOY_HOST" bash -s -- "$REMOTE_ROOT" "$sha" <<'REMOTE'
set -euo pipefail
root=$1
sha=$2
[[ "$root" == /var/www/keychron.karti.ai && "$sha" =~ ^[0-9a-f]{40}$ ]] || exit 2
release="$root/releases/$sha"
[[ -d "$release" && ! -L "$release" ]] || { echo "release not found: $sha" >&2; exit 1; }
if [[ -n "$(find "$release" -type l -print -quit)" ]] ||
  [[ -n "$(find "$release" ! -type d ! -type f -print -quit)" ]]; then
  echo "release contains a symlink or non-regular entry: $sha" >&2
  exit 1
fi
cd "$release"
sha256sum --check --strict SHA256SUMS
node -e '
  const fs = require("fs");
  const expected = process.argv[1];
  const manifest = JSON.parse(fs.readFileSync("release.json", "utf8"));
  if (
    manifest.schema !== "keysmith.public-release/v1" ||
    manifest.source?.commit !== expected ||
    !/^[0-9a-f]{64}$/.test(manifest.deployment?.caddy_sha256 ?? "")
  ) process.exit(1);
' "$sha"
config="$root/configs/$sha.caddy"
[[ -f "$config" && ! -L "$config" ]] || { echo "release Caddy fragment not found: $sha" >&2; exit 1; }
expected_caddy=$(node -e '
  const fs = require("fs");
  console.log(JSON.parse(fs.readFileSync(process.argv[1], "utf8")).deployment.caddy_sha256);
' "$release/release.json")
actual_caddy=$(sha256sum "$config" | awk '{print $1}')
[[ "$actual_caddy" == "$expected_caddy" ]] || { echo "release Caddy fragment checksum mismatch: $sha" >&2; exit 1; }
REMOTE
}

remote_current() {
  ssh -o BatchMode=yes "$DEPLOY_HOST" bash -s -- "$REMOTE_ROOT" <<'REMOTE'
set -euo pipefail
root=$1
[[ "$root" == /var/www/keychron.karti.ai ]] || exit 2
if [[ -e "$root/current" && ! -L "$root/current" ]]; then
  echo "deploy: current is not a symlink" >&2
  exit 1
fi
target=$(readlink "$root/current" 2>/dev/null || true)
if [[ -n "$target" ]]; then
  [[ "$target" =~ ^releases/([0-9a-f]{40})$ ]] || { echo "deploy: unsafe current target" >&2; exit 1; }
  printf '%s\n' "${BASH_REMATCH[1]}"
fi
REMOTE
}

remote_previous() {
  ssh -o BatchMode=yes "$DEPLOY_HOST" bash -s -- "$REMOTE_ROOT" <<'REMOTE'
set -euo pipefail
root=$1
[[ "$root" == /var/www/keychron.karti.ai ]] || exit 2
if [[ -e "$root/previous" && ! -L "$root/previous" ]]; then
  echo "deploy: previous is not a symlink" >&2
  exit 1
fi
target=$(readlink "$root/previous" 2>/dev/null || true)
if [[ -n "$target" ]]; then
  [[ "$target" =~ ^releases/([0-9a-f]{40})$ ]] || { echo "deploy: unsafe previous target" >&2; exit 1; }
  printf '%s\n' "${BASH_REMATCH[1]}"
fi
REMOTE
}

remote_activate() {
  local sha=$1
  ssh -o BatchMode=yes "$DEPLOY_HOST" bash -s -- "$REMOTE_ROOT" "$sha" <<'REMOTE'
set -euo pipefail
root=$1
sha=$2
[[ "$root" == /var/www/keychron.karti.ai && "$sha" =~ ^[0-9a-f]{40}$ ]] || exit 2
[[ -d "$root/releases/$sha" && ! -L "$root/releases/$sha" ]] || exit 1
if [[ -e "$root/current" && ! -L "$root/current" ]]; then
  echo "deploy: current is not a symlink" >&2
  exit 1
fi
if [[ -e "$root/previous" && ! -L "$root/previous" ]]; then
  echo "deploy: previous is not a symlink" >&2
  exit 1
fi
current=$(readlink "$root/current" 2>/dev/null || true)
if [[ -n "$current" && "$current" != "releases/$sha" ]]; then
  [[ "$current" =~ ^releases/[0-9a-f]{40}$ ]] || exit 1
  ln -s -- "$current" "$root/previous.next"
  mv -Tf -- "$root/previous.next" "$root/previous"
fi
ln -s -- "releases/$sha" "$root/current.next"
mv -Tf -- "$root/current.next" "$root/current"
REMOTE
}

remote_restore_pointer() {
  local old_sha=$1
  local old_previous=$2
  ssh -o BatchMode=yes "$DEPLOY_HOST" bash -s -- "$REMOTE_ROOT" "$old_sha" "$old_previous" <<'REMOTE'
set -euo pipefail
root=$1
sha=$2
previous=$3
[[ "$root" == /var/www/keychron.karti.ai ]] || exit 2
if [[ -e "$root/current" && ! -L "$root/current" ]]; then
  echo "deploy: current is not a symlink" >&2
  exit 1
fi
if [[ -e "$root/previous" && ! -L "$root/previous" ]]; then
  echo "deploy: previous is not a symlink" >&2
  exit 1
fi
if [[ -n "$sha" ]]; then
  [[ "$sha" =~ ^[0-9a-f]{40}$ && -d "$root/releases/$sha" && ! -L "$root/releases/$sha" ]] || exit 2
fi
if [[ -n "$previous" ]]; then
  [[ "$previous" =~ ^[0-9a-f]{40}$ && -d "$root/releases/$previous" && ! -L "$root/releases/$previous" ]] || exit 2
fi
if [[ -z "$sha" ]]; then
  if [[ -L "$root/current" ]]; then rm -- "$root/current"; fi
else
  ln -s -- "releases/$sha" "$root/current.restore"
  mv -Tf -- "$root/current.restore" "$root/current"
fi
if [[ -z "$previous" ]]; then
  if [[ -L "$root/previous" ]]; then rm -- "$root/previous"; fi
else
  ln -s -- "releases/$previous" "$root/previous.restore"
  mv -Tf -- "$root/previous.restore" "$root/previous"
fi
REMOTE
}

install_caddy_for() {
  local sha=$1
  local config="$CONFIGS/$sha.caddy"
  local control="$REMOTE_ROOT/.install-caddy-$sha-$$.sh"
  if ssh -o BatchMode=yes "$DEPLOY_HOST" test -e "$control" ||
    ssh -o BatchMode=yes "$DEPLOY_HOST" test -L "$control"; then
    echo "deploy: refusing existing remote control path: $control" >&2
    return 1
  fi
  scp -q -- "$SCRIPT_DIR/install-caddy.sh" "$DEPLOY_HOST:$control"
  if ! ssh -o BatchMode=yes "$DEPLOY_HOST" bash "$control" "$config"; then
    echo "deploy: Caddy installation failed; remote control script retained at $control" >&2
    return 1
  fi
  ssh -o BatchMode=yes "$DEPLOY_HOST" rm -- "$control"
}

restore_last_caddy() {
  local control="$REMOTE_ROOT/.restore-caddy-$$.sh"
  if ssh -o BatchMode=yes "$DEPLOY_HOST" test -e "$control" ||
    ssh -o BatchMode=yes "$DEPLOY_HOST" test -L "$control"; then
    echo "deploy: refusing existing remote restore path: $control" >&2
    return 1
  fi
  scp -q -- "$SCRIPT_DIR/install-caddy.sh" "$DEPLOY_HOST:$control"
  if ! ssh -o BatchMode=yes "$DEPLOY_HOST" bash "$control" --restore-last; then
    echo "deploy: exact Caddy restore failed; remote control script retained at $control" >&2
    return 1
  fi
  ssh -o BatchMode=yes "$DEPLOY_HOST" rm -- "$control"
}

restore_deployment() {
  local old_sha=$1
  local old_previous=$2
  remote_restore_pointer "$old_sha" "$old_previous"
  restore_last_caddy
}

upload_release() {
  local sha=$1
  if ssh -o BatchMode=yes "$DEPLOY_HOST" test -d "$RELEASES/$sha"; then
    echo "==> immutable release already exists: $sha"
    local local_sums_hash remote_sums_hash
    local_sums_hash=$(sha256sum "$DIST_DIR/SHA256SUMS" | awk '{print $1}')
    remote_sums_hash=$(ssh -o BatchMode=yes "$DEPLOY_HOST" sha256sum "$RELEASES/$sha/SHA256SUMS" | awk '{print $1}')
    [[ "$local_sums_hash" == "$remote_sums_hash" ]] || {
      echo "deploy: immutable release $sha differs from the exact local build" >&2
      exit 1
    }
  else
    local stage="$REMOTE_ROOT/.incoming-$sha-$$"
    echo "==> uploading release $sha"
    ssh -o BatchMode=yes "$DEPLOY_HOST" mkdir -- "$stage"
    rsync -a --checksum -- "$DIST_DIR/" "$DEPLOY_HOST:$stage/"
    ssh -o BatchMode=yes "$DEPLOY_HOST" bash -s -- "$stage" "$RELEASES/$sha" "$sha" <<'REMOTE'
set -euo pipefail
stage=$1
release=$2
sha=$3
[[ "$stage" =~ ^/var/www/keychron\.karti\.ai/\.incoming-[0-9a-f]{40}-[0-9]+$ ]] || exit 2
[[ "$release" == "/var/www/keychron.karti.ai/releases/$sha" && "$sha" =~ ^[0-9a-f]{40}$ ]] || exit 2
[[ -d "$stage" && ! -L "$stage" && ! -e "$release" ]] || exit 1
if [[ -n "$(find "$stage" -type l -print -quit)" ]] ||
  [[ -n "$(find "$stage" ! -type d ! -type f -print -quit)" ]]; then
  echo "staged release contains a symlink or non-regular entry" >&2
  exit 1
fi
cd "$stage"
sha256sum --check --strict SHA256SUMS
node -e '
  const fs = require("fs");
  const expected = process.argv[1];
  const manifest = JSON.parse(fs.readFileSync("release.json", "utf8"));
  if (manifest.schema !== "keysmith.public-release/v1" || manifest.source?.commit !== expected) process.exit(1);
' "$sha"
find "$stage" -type f -exec chmod 0444 {} +
find "$stage" -type d -exec chmod 0555 {} +
cd /
mv -- "$stage" "$release"
REMOTE
  fi

  local remote_config="$CONFIGS/$sha.caddy"
  if ssh -o BatchMode=yes "$DEPLOY_HOST" test -e "$remote_config" ||
    ssh -o BatchMode=yes "$DEPLOY_HOST" test -L "$remote_config"; then
    if ! ssh -o BatchMode=yes "$DEPLOY_HOST" test -f "$remote_config" ||
      ssh -o BatchMode=yes "$DEPLOY_HOST" test -L "$remote_config"; then
      echo "deploy: preserved Caddy config has an unexpected type: $remote_config" >&2
      exit 1
    fi
    local local_hash remote_hash
    local_hash=$(sha256sum "$SCRIPT_DIR/Caddyfile" | awk '{print $1}')
    remote_hash=$(ssh -o BatchMode=yes "$DEPLOY_HOST" sha256sum "$remote_config" | awk '{print $1}')
    [[ "$local_hash" == "$remote_hash" ]] || {
      echo "deploy: immutable Caddy config for $sha already exists with another hash" >&2
      exit 1
    }
  else
    scp -q -- "$SCRIPT_DIR/Caddyfile" "$DEPLOY_HOST:$remote_config"
    ssh -o BatchMode=yes "$DEPLOY_HOST" chmod 0444 "$remote_config"
  fi
  remote_verify_release "$sha"
}

build_release() {
  local sha=$1
  cd "$REPO"
  [[ "$(git rev-parse HEAD)" == "$sha" ]] || {
    echo "deploy: requested SHA is not the checked-out HEAD" >&2
    exit 1
  }
  git cat-file -e "$sha^{commit}"
  if [[ -n "$(git status --porcelain --untracked-files=normal)" ]]; then
    echo "deploy: exact releases require a clean working tree" >&2
    exit 1
  fi

  echo "==> installing and auditing public-site dependencies"
  npm --prefix "$SITE_DIR" ci
  npm --prefix "$SITE_DIR" audit --audit-level=high
  echo "==> building apps/site at $sha"
  npm --prefix "$SITE_DIR" run build
  node "$SCRIPT_DIR/generate-release-manifest.mjs" --dir "$DIST_DIR" --source-sha "$sha"
  node "$SCRIPT_DIR/check-static.mjs" --dir "$DIST_DIR" --sha "$sha"
}

public_smoke() {
  local sha=$1
  local attempt
  for attempt in 1 2 3 4 5; do
    if node "$SCRIPT_DIR/check-static.mjs" --url "$PUBLIC_URL" --sha "$sha"; then
      return 0
    fi
    if (( attempt < 5 )); then
      echo "deploy: public smoke attempt $attempt/5 failed; retrying in 3 seconds" >&2
      sleep 3
    fi
  done
  return 1
}

if [[ "$command" == status ]]; then
  current=$(remote_current)
  previous=$(remote_previous)
  printf 'current  : %s\n' "${current:-none}"
  printf 'previous : %s\n' "${previous:-none}"
  ssh -o BatchMode=yes "$DEPLOY_HOST" sudo caddy validate --config /etc/caddy/Caddyfile
  exit 0
fi

if [[ "$command" == deploy ]]; then
  remote_bootstrap
  acquire_deploy_lock
  build_release "$source_sha"
  upload_release "$source_sha"
else
  acquire_deploy_lock
  remote_verify_release "$source_sha"
  ssh -o BatchMode=yes "$DEPLOY_HOST" test -f "$CONFIGS/$source_sha.caddy" || {
    echo "deploy: release has no preserved Caddy fragment: $source_sha" >&2
    exit 1
  }
fi

old_sha=$(remote_current)
old_previous=$(remote_previous)
if [[ "$old_sha" == "$source_sha" ]]; then
  echo "==> release $source_sha is already current; revalidating"
else
  echo "==> atomically activating $source_sha (previous: ${old_sha:-none})"
  remote_activate "$source_sha"
fi

if ! install_caddy_for "$source_sha"; then
  # install-caddy.sh validates before mutation and performs its own exact
  # rollback if a failure occurs after mutation. Only restore the content
  # pointer here: invoking --restore-last after an early preflight failure
  # could otherwise consume state left by an older deployment transaction.
  remote_restore_pointer "$old_sha" "$old_previous"
  exit 1
fi

if ! public_smoke "$source_sha"; then
  echo "deploy: public smoke failed; restoring ${old_sha:-no prior release}" >&2
  restore_deployment "$old_sha" "$old_previous"
  if [[ -n "$old_sha" ]]; then
    public_smoke "$old_sha" || echo "deploy: previous release also failed public smoke" >&2
  fi
  exit 1
fi

echo "==> live: $PUBLIC_URL ($source_sha)"
