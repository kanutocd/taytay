#!/usr/bin/env bash
set -euo pipefail

# Runs the real Taytay -> Lunsaran -> Brutus path. The environment must provide
# a disposable stack and a device/workload credential already enrolled for the
# configured project; this harness never performs enrollment or prints secrets.

if [[ "${TAYTAY_CROSS_REPO:-0}" != "1" ]]; then
  echo "cross-repository acceptance skipped; set TAYTAY_CROSS_REPO=1 to enable"
  exit 0
fi

for command in cargo curl docker jq psql sha256sum stat awk sed rg; do
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
recovery_spool="$spool_dir/recovery"
recovery_artifact="$recovery_spool/recovery.artifact"
recovery_config="$recovery_spool/taytay.toml"
expired_token="crossrepo-expired-${RANDOM}-${RANDOM}.token"
revoked_token="crossrepo-revoked-${RANDOM}-${RANDOM}.token"
fixture_name="taytay-cross-repo-${RANDOM}-${RANDOM}"
expired_token_file="$spool_dir/expired.token"
revoked_token_file="$spool_dir/revoked.token"
api_response="$spool_dir/api-response.json"
postgres_container=""
fixture_cleanup=""

cleanup() {
  exit_status=$?
  set +e
  if [[ -n "$fixture_cleanup" && -n "$postgres_container" ]]; then
    docker exec "$postgres_container" psql -U lunsaran -d lunsaran_development -c "$fixture_cleanup" >/dev/null 2>&1
  fi
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
artifact_id="${TAYTAY_ACCEPTANCE_ARTIFACT_ID:-cross-repo-acceptance}"

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

request_session() {
  local token="$1"
  local project_id="$2"
  local idempotency_key="$3"
  local response_path="$4"
  curl -sS --max-time 15 -o "$response_path" -w '%{http_code}' \
    -X POST "$TAYTAY_LUNSARAN_BASE_URL/v1/upload-sessions" \
    -H "Authorization: Bearer $token" \
    -H 'Content-Type: application/json' \
    -H "Idempotency-Key: $idempotency_key" \
    --data "{\"project_id\":\"$project_id\",\"size\":32,\"filename\":\"cross-repo.artifact\",\"content_type\":\"text/plain\",\"checksum\":\"$checksum\",\"client_kind\":\"entregar\"}"
}

valid_token="$(<"$TAYTAY_LUNSARAN_TOKEN_FILE")"
idempotency_key="taytay-cross-repo-idempotency-$(date +%s%N)"
first_code="$(request_session "$valid_token" "$TAYTAY_PROJECT_ID" "$idempotency_key" "$api_response")"
first_id="$(jq -r '.id // empty' "$api_response")"
second_code="$(request_session "$valid_token" "$TAYTAY_PROJECT_ID" "$idempotency_key" "$api_response")"
second_id="$(jq -r '.id // empty' "$api_response")"
[[ "$first_code" =~ ^2[0-9][0-9]$ && "$second_code" =~ ^2[0-9][0-9]$ && -n "$first_id" && "$first_id" == "$second_id" ]] || {
  echo "idempotency replay did not reuse the upload session" >&2
  exit 1
}

mkdir -p "$recovery_spool/resume"
printf '%s\n' 'taytay-cross-repository-restart-payload' >"$recovery_artifact"
recovery_checksum="$(sha256sum "$recovery_artifact" | awk '{print $1}')"
recovery_size="$(stat -c '%s' "$recovery_artifact")"
recovery_path="$(printf '%s' "$recovery_artifact" | sed 's/\\/\\\\/g; s/"/\\"/g')"
printf '%s\n' \
  "spool_dir = \"$recovery_spool\"" \
  'quota_bytes = 1048576' \
  'upload_workers = 1' \
  "lunsaran_base_url = \"$TAYTAY_LUNSARAN_BASE_URL\"" \
  "organization_id = \"$TAYTAY_ORGANIZATION_ID\"" \
  "project_id = \"$TAYTAY_PROJECT_ID\"" \
  "token_file = \"$TAYTAY_LUNSARAN_TOKEN_FILE\"" >"$recovery_config"
printf '%s\n' "{\"artifact\":{\"id\":\"cross-repo-restart-$(date +%s%N)\",\"source\":\"cross-repo\",\"path\":\"$recovery_path\",\"media_type\":\"text/plain\",\"captured_at\":null,\"metadata\":{},\"size\":$recovery_size,\"checksum_sha256\":\"$recovery_checksum\"},\"state\":\"Uploading\",\"attempts\":1,\"server_url\":null,\"offset\":0,\"last_error\":null}" >"$recovery_spool/ledger.jsonl"
target/debug/taytay "$recovery_config" >"$spool_dir/recovery.out" 2>&1
rg -q '"completed_jobs":1,"failed_jobs":0' "$spool_dir/recovery.out"
rg -q '"state":"Completed"' "$recovery_spool/ledger.jsonl"

invalid_token="crossrepo-invalid-${RANDOM}-${RANDOM}.token"
invalid_code="$(request_session "$invalid_token" "$TAYTAY_PROJECT_ID" "taytay-cross-repo-invalid-$(date +%s%N)" "$api_response")"
[[ "$invalid_code" == "401" || "$invalid_code" == "403" ]] || {
  echo "invalid credential was not rejected safely" >&2
  exit 1
}

ineligible_project="00000000-0000-0000-0000-000000000099"
ineligible_code="$(request_session "$valid_token" "$ineligible_project" "taytay-cross-repo-ineligible-$(date +%s%N)" "$api_response")"
[[ "$ineligible_code" == "403" || "$ineligible_code" == "404" ]] || {
  echo "ineligible project was not rejected safely" >&2
  exit 1
}

if [[ "$use_existing_stack" == "1" ]]; then
  postgres_container="${TAYTAY_POSTGRES_CONTAINER:-infra-postgres-1}"
  device_id="$(docker exec "$postgres_container" psql -U lunsaran -d lunsaran_development -Atc "SELECT device_id::text FROM api_keys WHERE organization_id = '$TAYTAY_ORGANIZATION_ID' AND revoked_at IS NULL AND device_id IS NOT NULL ORDER BY created_at DESC LIMIT 1")"
  [[ "$device_id" =~ ^[0-9a-f-]{36}$ ]] || {
    echo "could not resolve a device for disposable credential fixtures" >&2
    exit 1
  }
  printf '%s\n' "$expired_token" >"$expired_token_file"
  printf '%s\n' "$revoked_token" >"$revoked_token_file"
  chmod 600 "$expired_token_file" "$revoked_token_file"
  docker exec "$postgres_container" psql -U lunsaran -d lunsaran_development -v ON_ERROR_STOP=1 -c \
    "INSERT INTO api_keys (organization_id, device_id, name, key_hash, key_prefix, scopes, expires_at, revoked_at) VALUES ('$TAYTAY_ORGANIZATION_ID'::uuid, '$device_id'::uuid, '$fixture_name-expired', digest('$expired_token','sha256'), split_part('$expired_token','.',1), ARRAY['upload_sessions:create'], now() - interval '1 minute', NULL), ('$TAYTAY_ORGANIZATION_ID'::uuid, '$device_id'::uuid, '$fixture_name-revoked', digest('$revoked_token','sha256'), split_part('$revoked_token','.',1), ARRAY['upload_sessions:create'], now() + interval '1 hour', now())" >/dev/null
  fixture_cleanup="DELETE FROM api_keys WHERE organization_id = '$TAYTAY_ORGANIZATION_ID' AND name IN ('$fixture_name-expired', '$fixture_name-revoked')"
  expired_code="$(request_session "$expired_token" "$TAYTAY_PROJECT_ID" "taytay-cross-repo-expired-$(date +%s%N)" "$api_response")"
  revoked_code="$(request_session "$revoked_token" "$TAYTAY_PROJECT_ID" "taytay-cross-repo-revoked-$(date +%s%N)" "$api_response")"
  [[ "$expired_code" == "401" || "$expired_code" == "403" ]] || { echo "expired credential was not rejected safely" >&2; exit 1; }
  [[ "$revoked_code" == "401" || "$revoked_code" == "403" ]] || { echo "revoked credential was not rejected safely" >&2; exit 1; }
  docker exec "$postgres_container" psql -U lunsaran -d lunsaran_development -v ON_ERROR_STOP=1 -c "$fixture_cleanup" >/dev/null
fi

echo "cross-repository acceptance passed: session authorization, TUS completion, checksum-bound artifact, durable completion, idempotency replay, restart recovery, invalid/project authorization, and credential rejection"
