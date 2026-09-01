#!/usr/bin/env bash
# Install one reviewed static-site fragment into cloud-2's shared Caddy config.
# This script is copied to cloud-2 and invoked by deploy.sh; it never restarts
# Caddy and rolls both files back if validation or reload fails.
set -euo pipefail

MAIN=/etc/caddy/Caddyfile
LIVE_FRAGMENT=/etc/caddy/keychron.karti.ai.caddy
IMPORT_LINE="import $LIVE_FRAGMENT"
STATE_DIR=/var/lib/keysmith-public-deploy
CANDIDATE_FRAGMENT=${1:-}

scratch=$(mktemp -d /tmp/keysmith-caddy.XXXXXX)
cleanup() {
  rm -rf -- "$scratch"
}
trap cleanup EXIT

restore_last() {
  if ! sudo test -f "$MAIN" || sudo test -L "$MAIN"; then
    echo "install-caddy: $MAIN is not a regular file during restore" >&2
    exit 1
  fi
  if ! sudo test -f "$STATE_DIR/main.before" || sudo test -L "$STATE_DIR/main.before" ||
    ! sudo test -f "$STATE_DIR/had-fragment" || sudo test -L "$STATE_DIR/had-fragment"; then
    echo "install-caddy: no exact pre-change state is available" >&2
    exit 1
  fi

  local had_before had_current=0
  had_before=$(sudo cat "$STATE_DIR/had-fragment")
  [[ "$had_before" == 0 || "$had_before" == 1 ]] || {
    echo "install-caddy: invalid rollback state" >&2
    exit 1
  }

  sudo cp -- "$STATE_DIR/main.before" "$scratch/main.restore"
  sudo chown "$(id -u):$(id -g)" "$scratch/main.restore"
  cp -- "$MAIN" "$scratch/main.current"
  if [[ -f "$LIVE_FRAGMENT" && ! -L "$LIVE_FRAGMENT" ]]; then
    cp -- "$LIVE_FRAGMENT" "$scratch/fragment.current"
    had_current=1
  elif [[ -e "$LIVE_FRAGMENT" || -L "$LIVE_FRAGMENT" ]]; then
    echo "install-caddy: refusing unexpected live fragment type during restore" >&2
    exit 1
  fi

  if [[ "$had_before" == 1 ]]; then
    if ! sudo test -f "$STATE_DIR/fragment.before" || sudo test -L "$STATE_DIR/fragment.before"; then
      echo "install-caddy: rollback fragment is missing" >&2
      exit 1
    fi
    sudo cp -- "$STATE_DIR/fragment.before" "$scratch/fragment.restore"
    sudo chown "$(id -u):$(id -g)" "$scratch/fragment.restore"
    sed "s#^${IMPORT_LINE}\$#import $scratch/fragment.restore#" \
      "$scratch/main.restore" > "$scratch/main.restore.candidate"
  else
    cp -- "$scratch/main.restore" "$scratch/main.restore.candidate"
  fi
  sudo caddy validate --config "$scratch/main.restore.candidate"

  restore_failed_state() {
    echo "install-caddy: exact restore failed; returning to the configuration active before restore" >&2
    sudo install -o root -g root -m 0644 "$scratch/main.current" "$MAIN"
    if (( had_current == 1 )); then
      sudo install -o root -g root -m 0644 "$scratch/fragment.current" "$LIVE_FRAGMENT"
    else
      sudo rm -f -- "$LIVE_FRAGMENT"
    fi
    sudo caddy validate --config "$MAIN"
    sudo systemctl reload caddy
  }

  sudo install -o root -g root -m 0644 "$scratch/main.restore" "$MAIN"
  if [[ "$had_before" == 1 ]]; then
    sudo install -o root -g root -m 0644 "$scratch/fragment.restore" "$LIVE_FRAGMENT"
  else
    sudo rm -f -- "$LIVE_FRAGMENT"
  fi
  if ! sudo caddy validate --config "$MAIN"; then
    restore_failed_state
    exit 1
  fi
  if ! sudo systemctl reload caddy || ! systemctl is-active --quiet caddy; then
    restore_failed_state
    exit 1
  fi
  echo "install-caddy: restored the exact pre-change Caddy configuration"
}

if [[ "$CANDIDATE_FRAGMENT" == --restore-last ]]; then
  restore_last
  exit 0
fi

if [[ ! "$CANDIDATE_FRAGMENT" =~ ^/var/www/keychron\.karti\.ai/configs/[0-9a-f]{40}\.caddy$ ]]; then
  echo "install-caddy: candidate must be a versioned Keychron fragment under /var/www" >&2
  exit 2
fi
if [[ ! -f "$CANDIDATE_FRAGMENT" || -L "$CANDIDATE_FRAGMENT" ]]; then
  echo "install-caddy: candidate is not a regular file: $CANDIDATE_FRAGMENT" >&2
  exit 1
fi
if grep -Eq '^[[:space:]]*reverse_proxy\b' "$CANDIDATE_FRAGMENT"; then
  echo "install-caddy: public Keysmith fragment must not contain reverse_proxy" >&2
  exit 1
fi
for required in "keychron.karti.ai" "connect-src 'none'" "respond @api 404" "respond @non_read 405"; do
  if ! grep -Fq "$required" "$CANDIDATE_FRAGMENT"; then
    echo "install-caddy: candidate is missing required boundary: $required" >&2
    exit 1
  fi
done
if [[ ! -f "$MAIN" || -L "$MAIN" ]]; then
  echo "install-caddy: $MAIN is not a regular file" >&2
  exit 1
fi
if grep -Eq '^[[:space:]]*keychron\.karti\.ai[[:space:]]*\{' "$MAIN"; then
  echo "install-caddy: refusing to compete with an inline keychron.karti.ai block" >&2
  exit 1
fi

import_count=$(grep -Fxc "$IMPORT_LINE" "$MAIN" || true)
if (( import_count > 1 )); then
  echo "install-caddy: duplicate Keychron import lines in $MAIN" >&2
  exit 1
fi

main_before="$scratch/Caddyfile.before"
fragment_before="$scratch/keychron.before"
candidate_main="$scratch/Caddyfile.candidate"
main_next="$scratch/Caddyfile.next"
cp -- "$MAIN" "$main_before"
had_fragment=0
if [[ -f "$LIVE_FRAGMENT" && ! -L "$LIVE_FRAGMENT" ]]; then
  cp -- "$LIVE_FRAGMENT" "$fragment_before"
  had_fragment=1
elif [[ -e "$LIVE_FRAGMENT" || -L "$LIVE_FRAGMENT" ]]; then
  echo "install-caddy: refusing unexpected live fragment type" >&2
  exit 1
fi

# Validate the candidate fragment in the context of every existing shared
# snippet, without changing either production file.
if (( import_count == 1 )); then
  sed "s#^${IMPORT_LINE}\$#import ${CANDIDATE_FRAGMENT}#" "$MAIN" > "$candidate_main"
else
  cp -- "$MAIN" "$candidate_main"
  printf '\n# Keysmith public static site (managed by its guarded deploy script)\nimport %s\n' \
    "$CANDIDATE_FRAGMENT" >> "$candidate_main"
fi
sudo caddy validate --config "$candidate_main"

# Persist the exact pre-change pair outside the webroot. A later public-smoke
# failure can therefore undo a successful reload, including removing the
# fragment/import on a failed first launch.
sudo install -d -o root -g root -m 0700 "$STATE_DIR"
sudo install -o root -g root -m 0600 "$main_before" "$STATE_DIR/main.before"
printf '%s\n' "$had_fragment" | sudo tee "$STATE_DIR/had-fragment" >/dev/null
sudo chmod 0600 "$STATE_DIR/had-fragment"
if (( had_fragment == 1 )); then
  sudo install -o root -g root -m 0600 "$fragment_before" "$STATE_DIR/fragment.before"
else
  sudo rm -f -- "$STATE_DIR/fragment.before"
fi

restore_previous() {
  echo "install-caddy: restoring previous Caddy files" >&2
  sudo install -o root -g root -m 0644 "$main_before" "$MAIN"
  if (( had_fragment == 1 )); then
    sudo install -o root -g root -m 0644 "$fragment_before" "$LIVE_FRAGMENT"
  else
    sudo rm -f -- "$LIVE_FRAGMENT"
  fi
  sudo caddy validate --config "$MAIN"
  sudo systemctl reload caddy
}

sudo install -o root -g root -m 0644 "$CANDIDATE_FRAGMENT" "$LIVE_FRAGMENT"
if (( import_count == 0 )); then
  cp -- "$MAIN" "$main_next"
  printf '\n# Keysmith public static site (managed by its guarded deploy script)\n%s\n' \
    "$IMPORT_LINE" >> "$main_next"
  sudo install -o root -g root -m 0644 "$main_next" "$MAIN"
fi

if ! sudo caddy validate --config "$MAIN"; then
  restore_previous
  exit 1
fi
if ! sudo systemctl reload caddy || ! systemctl is-active --quiet caddy; then
  restore_previous
  exit 1
fi

echo "install-caddy: validated and reloaded Caddy; fragment $(sha256sum "$LIVE_FRAGMENT" | awk '{print $1}')"
