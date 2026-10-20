#!/usr/bin/env bash
set -euo pipefail

# Runs the real Taytay -> Lunsaran -> Brutus path. The environment must provide
# a disposable stack and a device/workload credential already enrolled for the
# configured project; this harness never performs enrollment or prints secrets.

if [[ "${TAYTAY_CROSS_REPO:-0}" != "1" ]]; then
  echo "cross-repository acceptance skipped; set TAYTAY_CROSS_REPO=1 to enable"
  exit 0
fi

for command in cargo docker sha256sum stat awk sed rg; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "cross-repository acceptance requires: $command" >&2
    exit 1
  }
done

: "${TAYTAY_LUNSARAN_BASE_URL:?set TAYTAY_LUNSARAN_BASE_URL to the disposable API origin}"
: "${TAYTAY_LUNSARAN_TOKEN_FILE:?set TAYTAY_LUNSARAN_TOKEN_FILE to a mode-600 provisioned device credential}"
: "${TAYTAY_ORGANIZATION_ID:?set TAYTAY_ORGANIZATION_ID to the enrolled organization UUID}"
: "${TAYTAY_PROJECT_ID:?set TAYTAY_PROJECT_ID to the enrolled project UUID}"
: "${TAYTAY_LUNSARAN_COMPOSE_FILES:?set TAYTAY_LUNSARAN_COMPOSE_FILES to the Lunsaran compose files, in order}"
: "${TAYTAY_BRUTUS_COMPOSE_FILE:?set TAYTAY_BRUTUS_COMPOSE_FILE to the Brutus compose file}"

read -r -a lunsaran_compose_paths <<<"$TAYTAY_LUNSARAN_COMPOSE_FILES"
lunsaran_compose_args=()
for compose_file in "${lunsaran_compose_paths[@]}"; do
  [[ -f "$compose_file" ]] || {
    echo "Lunsaran compose file does not exist: $compose_file" >&2
    exit 1
  }
  lunsaran_compose_args+=(-f "$compose_file")
done
[[ -f "$TAYTAY_BRUTUS_COMPOSE_FILE" ]] || {
  echo "Brutus compose file does not exist: $TAYTAY_BRUTUS_COMPOSE_FILE" >&2
  exit 1
}

[[ -f "$TAYTAY_LUNSARAN_TOKEN_FILE" ]] || {
  echo "credential file does not exist" >&2
  exit 1
}
[[ "$(stat -c '%a' "$TAYTAY_LUNSARAN_TOKEN_FILE")" == "600" ]] || {
  echo "credential file must have mode 600" >&2
  exit 1
}

compose_project="taytay-cross-repo-${RANDOM}-${RANDOM}"
use_existing_stack="${TAYTAY_USE_EXISTING_STACK:-0}"
spool_dir="$(mktemp -d "${TMPDIR:-/tmp}/taytay-cross-repo.XXXXXX")"
config_file="$spool_dir/taytay.toml"
artifact_file="$spool_dir/cross-repo.artifact"
output_file="$spool_dir/taytay.out"

cleanup() {
  exit_status=$?
  set +e
  if [[ "$use_existing_stack" != "1" && "${TAYTAY_KEEP_CROSS_REPO_STACK:-0}" != "1" ]]; then
    docker compose -p "$compose_project" "${lunsaran_compose_args[@]}" down --volumes --remove-orphans >/dev/null 2>&1
    docker compose -p "$compose_project" -f "$TAYTAY_BRUTUS_COMPOSE_FILE" down --volumes --remove-orphans >/dev/null 2>&1
  fi
  if [[ "$exit_status" -ne 0 || "${TAYTAY_KEEP_CROSS_REPO_ARTIFACTS:-0}" == "1" ]]; then
    echo "cross-repository acceptance artifacts preserved at $spool_dir" >&2
  else
    rm -rf "$spool_dir"
  fi
  exit "$exit_status"
}
trap cleanup EXIT INT TERM

if [[ "$use_existing_stack" != "1" ]]; then
  docker compose -p "$compose_project" "${lunsaran_compose_args[@]}" up -d --wait
  docker compose -p "$compose_project" -f "$TAYTAY_BRUTUS_COMPOSE_FILE" up -d --wait
fi

printf 'taytay-cross-repository-payload\n' >"$artifact_file"
checksum="$(sha256sum "$artifact_file" | awk '{print $1}')"
artifact_size="$(stat -c '%s' "$artifact_file")"
artifact_path="$(printf '%s' "$artifact_file" | sed 's/\\/\\\\/g; s/"/\\"/g')"
artifact_id="cross-repo-acceptance"

printf '%s\n' \
  "spool_dir = \"$spool_dir\"" \
  'quota_bytes = 1048576' \
  'upload_workers = 1' \
  "lunsaran_base_url = \"$TAYTAY_LUNSARAN_BASE_URL\"" \
  "organization_id = \"$TAYTAY_ORGANIZATION_ID\"" \
  "project_id = \"$TAYTAY_PROJECT_ID\"" \
  "token_file = \"$TAYTAY_LUNSARAN_TOKEN_FILE\"" >"$config_file"

printf '%s\n' "{\"artifact\":{\"id\":\"$artifact_id\",\"source\":\"cross-repo\",\"path\":\"$artifact_path\",\"media_type\":\"text/plain\",\"captured_at\":null,\"metadata\":{},\"size\":$artifact_size,\"checksum_sha256\":\"$checksum\"},\"state\":\"Published\",\"attempts\":0,\"server_url\":null,\"offset\":0,\"last_error\":null}" >"$spool_dir/ledger.jsonl"

cargo build -p lunsaran-taytay --bin taytay --quiet
if ! target/debug/taytay "$config_file" >"$output_file" 2>&1; then
  echo "taytay runtime failed; redacted output follows:" >&2
  sed -E 's/(Bearer )[A-Za-z0-9._~+\/-]+/\1[REDACTED]/g' "$output_file" >&2
  exit 1
fi

rg -q '"completed_jobs":1,"failed_jobs":0' "$output_file" || {
  echo "cross-repository upload did not report completion" >&2
  exit 1
}
rg -q '"state":"Completed"' "$spool_dir/ledger.jsonl" || {
  echo "cross-repository upload did not persist completion" >&2
  exit 1
}
if rg -n -F -- "$checksum" "$output_file"; then
  echo "checksum leaked to runtime output" >&2
  exit 1
fi
if rg -n -F -- "$(<"$TAYTAY_LUNSARAN_TOKEN_FILE")" "$output_file"; then
  echo "credential leaked to runtime output" >&2
  exit 1
fi

echo "cross-repository acceptance passed: session authorization, TUS completion, checksum-bound artifact, durable completion, and output redaction"
