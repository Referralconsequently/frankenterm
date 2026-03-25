#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG_DIR="$ROOT_DIR/tests/e2e/logs"
mkdir -p "$LOG_DIR"

run_id="replay_capture_extraction_$(date -u +%Y%m%dT%H%M%SZ)"
scenario_id="runtime_replay_capture_adapter"
json_log="$LOG_DIR/${run_id}.jsonl"
raw_log="$LOG_DIR/${run_id}.cargo.log"
cargo_home="/tmp/cargo-home-replay-capture-e2e"
component="replay_capture_extraction"
local_tmpdir="${FT_REPLAY_CAPTURE_LOCAL_TMPDIR:-${TMPDIR:-/tmp}}"
remote_tmpdir="${FT_REPLAY_CAPTURE_REMOTE_TMPDIR:-/home/ubuntu}"
cargo_target_dir="${FT_REPLAY_CAPTURE_TARGET_DIR:-$remote_tmpdir/target-replay-capture-e2e-${run_id}}"

RCH_FAIL_OPEN_REGEX='\[RCH\][[:space:]]+local|Remote execution failed: .*running locally|running locally|Failed to connect to ubuntu@|too long for Unix domain socket'
RCH_DAEMON_STATUS_LOG="${LOG_DIR}/${run_id}.rch_daemon_status.json"
RCH_DAEMON_START_LOG="${LOG_DIR}/${run_id}.rch_daemon_start.json"
RCH_STEP_TIMEOUT_SECS="${RCH_STEP_TIMEOUT_SECS:-900}"
TIMEOUT_BIN=""

now_ts() {
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

log_json() {
  echo "$1" >>"$json_log"
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"prereq_check\",\"status\":\"failed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{\"command\":\"$cmd\"},\"outcome\":\"failed\",\"reason_code\":\"missing_prerequisite\",\"error_code\":\"E2E-PREREQ\",\"artifact_path\":\"${json_log#"$ROOT_DIR"/}\"}"
    echo "missing required command: $cmd" >&2
    exit 1
  fi
}

probe_rch_workers() {
  local probe_log="$LOG_DIR/${run_id}.rch_probe.json"
  local probe_json

  log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"rch_probe\",\"status\":\"running\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{},\"outcome\":\"running\",\"reason_code\":null,\"error_code\":null,\"artifact_path\":\"${probe_log#"$ROOT_DIR"/}\"}"

  set +e
  env TMPDIR="$local_tmpdir" rch workers probe --all --json >"$probe_log" 2>&1
  local probe_rc=$?
  set -e

  if [[ $probe_rc -ne 0 ]]; then
    log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"rch_probe\",\"status\":\"failed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{},\"outcome\":\"failed\",\"reason_code\":\"rch_probe_failed\",\"error_code\":\"RCH-E100\",\"artifact_path\":\"${probe_log#"$ROOT_DIR"/}\"}"
    echo "rch workers probe failed" >&2
    exit 2
  fi

  probe_json="$(awk 'capture || /^[[:space:]]*[{]/{capture=1; print}' "$probe_log")"
  local healthy_workers
  healthy_workers="$(printf '%s\n' "$probe_json" | jq '[.data[]? | select(.status == "ok" or .status == "healthy" or .status == "reachable")] | length' 2>/dev/null || echo 0)"
  if [[ "$healthy_workers" -lt 1 ]]; then
    log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"rch_probe\",\"status\":\"failed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{\"healthy_workers\":$healthy_workers},\"outcome\":\"failed\",\"reason_code\":\"rch_workers_unreachable\",\"error_code\":\"RCH-E100\",\"artifact_path\":\"${probe_log#"$ROOT_DIR"/}\"}"
    echo "no reachable rch workers; refusing local fallback" >&2
    exit 2
  fi

  log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"rch_probe\",\"status\":\"passed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{\"healthy_workers\":$healthy_workers},\"outcome\":\"pass\",\"reason_code\":\"workers_reachable\",\"error_code\":null,\"artifact_path\":\"${probe_log#"$ROOT_DIR"/}\"}"
}

ensure_rch_daemon_running() {
  log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"rch_daemon_status\",\"status\":\"running\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{},\"outcome\":\"running\",\"reason_code\":null,\"error_code\":null,\"artifact_path\":\"${RCH_DAEMON_STATUS_LOG#"$ROOT_DIR"/}\"}"

  set +e
  env TMPDIR="$local_tmpdir" rch daemon status --json >"$RCH_DAEMON_STATUS_LOG" 2>&1
  local status_rc=$?
  set -e

  if [[ $status_rc -ne 0 ]]; then
    log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"rch_daemon_status\",\"status\":\"failed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{},\"outcome\":\"failed\",\"reason_code\":\"rch_daemon_status_failed\",\"error_code\":\"RCH-E101\",\"artifact_path\":\"${RCH_DAEMON_STATUS_LOG#"$ROOT_DIR"/}\"}"
    echo "rch daemon status failed" >&2
    exit 2
  fi

  if jq -e '.data.running == true' "$RCH_DAEMON_STATUS_LOG" >/dev/null 2>&1; then
    log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"rch_daemon_status\",\"status\":\"passed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{\"running\":true},\"outcome\":\"pass\",\"reason_code\":\"daemon_running\",\"error_code\":null,\"artifact_path\":\"${RCH_DAEMON_STATUS_LOG#"$ROOT_DIR"/}\"}"
    return 0
  fi

  log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"rch_daemon_status\",\"status\":\"passed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{\"running\":false},\"outcome\":\"pass\",\"reason_code\":\"daemon_not_running\",\"error_code\":null,\"artifact_path\":\"${RCH_DAEMON_STATUS_LOG#"$ROOT_DIR"/}\"}"
  log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"rch_daemon_start\",\"status\":\"running\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{},\"outcome\":\"running\",\"reason_code\":null,\"error_code\":null,\"artifact_path\":\"${RCH_DAEMON_START_LOG#"$ROOT_DIR"/}\"}"

  set +e
  env TMPDIR="$local_tmpdir" rch daemon start --json >"$RCH_DAEMON_START_LOG" 2>&1
  local start_rc=$?
  set -e

  if [[ $start_rc -ne 0 ]]; then
    log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"rch_daemon_start\",\"status\":\"failed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{},\"outcome\":\"failed\",\"reason_code\":\"rch_daemon_start_failed\",\"error_code\":\"RCH-E101\",\"artifact_path\":\"${RCH_DAEMON_START_LOG#"$ROOT_DIR"/}\"}"
    echo "rch daemon start failed" >&2
    exit 2
  fi

  sleep 2

  set +e
  env TMPDIR="$local_tmpdir" rch daemon status --json >"$RCH_DAEMON_STATUS_LOG" 2>&1
  status_rc=$?
  set -e

  if [[ $status_rc -ne 0 ]] || ! jq -e '.data.running == true' "$RCH_DAEMON_STATUS_LOG" >/dev/null 2>&1; then
    log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"rch_daemon_start\",\"status\":\"failed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{},\"outcome\":\"failed\",\"reason_code\":\"rch_daemon_unavailable\",\"error_code\":\"RCH-E101\",\"artifact_path\":\"${RCH_DAEMON_STATUS_LOG#"$ROOT_DIR"/}\"}"
    echo "rch daemon unavailable; refusing rch exec because it would fall back locally" >&2
    exit 2
  fi

  log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"rch_daemon_start\",\"status\":\"passed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"preflight\",\"inputs\":{\"running\":true},\"outcome\":\"pass\",\"reason_code\":\"daemon_started\",\"error_code\":null,\"artifact_path\":\"${RCH_DAEMON_STATUS_LOG#"$ROOT_DIR"/}\"}"
}

fatal() { echo "FATAL: $1" >&2; exit 1; }

run_rch() {
    TMPDIR=/tmp rch "$@"
}

capture_rch_queue_timeout_log() {
    local output_file="$1"
    local queue_log="${output_file%.log}.rch_queue_timeout.log"
    if ! run_rch queue >"${queue_log}" 2>&1; then
        queue_log="${output_file}"
    fi
    printf '%s\n' "${queue_log}"
}

resolve_timeout_bin() {
    if command -v timeout >/dev/null 2>&1; then
        TIMEOUT_BIN="timeout"
    elif command -v gtimeout >/dev/null 2>&1; then
        TIMEOUT_BIN="gtimeout"
    else
        TIMEOUT_BIN=""
    fi
}

check_rch_fallback() {
    local output_file="$1"
    if grep -Eq "${RCH_FAIL_OPEN_REGEX}" "${output_file}" 2>/dev/null; then
        fatal "rch fell back to local execution; refusing offload policy violation. See ${output_file}"
    fi
}

ensure_rch_ready() {
    resolve_timeout_bin
    if [[ -z "${TIMEOUT_BIN}" ]]; then
        fatal "timeout or gtimeout is required to fail closed on stalled remote execution."
    fi
    ensure_rch_daemon_running
}

assert_nonzero_tests_ran() {
    local output_file="$1"
    if grep -Eq 'running 0 tests|0 passed; 0 failed' "$output_file" 2>/dev/null; then
        fatal "cargo test completed without executing any tests. See ${output_file}"
    fi
}

log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"start\",\"status\":\"running\",\"correlation_id\":\"$run_id\",\"decision_path\":\"suite\",\"inputs\":{},\"outcome\":\"running\",\"reason_code\":null,\"error_code\":null,\"artifact_path\":\"${json_log#"$ROOT_DIR"/}\"}"

require_cmd jq
require_cmd rch
require_cmd cargo
probe_rch_workers
ensure_rch_ready

test_filter="runtime_with_replay_capture_adapter_shuts_down_cleanly"
cmd_str="env TMPDIR=$local_tmpdir ${TIMEOUT_BIN} --signal=TERM --kill-after=10 ${RCH_STEP_TIMEOUT_SECS} rch exec -- env TMPDIR=$remote_tmpdir CARGO_HOME=$cargo_home CARGO_TARGET_DIR=$cargo_target_dir cargo test -p frankenterm-core --lib $test_filter -- --nocapture"

log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"cargo_test\",\"status\":\"running\",\"correlation_id\":\"$run_id\",\"decision_path\":\"cargo_test\",\"inputs\":{\"command\":\"$cmd_str\",\"test\":\"$test_filter\"},\"outcome\":\"running\",\"reason_code\":null,\"error_code\":null,\"artifact_path\":\"${raw_log#"$ROOT_DIR"/}\"}"

set +e
(
  cd "${ROOT_DIR}"
  env TMPDIR="$local_tmpdir" "${TIMEOUT_BIN}" --signal=TERM --kill-after=10 "${RCH_STEP_TIMEOUT_SECS}" \
    rch exec -- env \
    "TMPDIR=$remote_tmpdir" \
    "CARGO_HOME=$cargo_home" \
    "CARGO_TARGET_DIR=$cargo_target_dir" \
    cargo test -p frankenterm-core --lib "$test_filter" -- --nocapture
) >"$raw_log" 2>&1
rc=$?
set -e

check_rch_fallback "$raw_log"

if [[ ${rc} -eq 124 || ${rc} -eq 137 ]]; then
  queue_log="$(capture_rch_queue_timeout_log "$raw_log")"
  log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"cargo_test\",\"status\":\"failed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"cargo_test\",\"inputs\":{\"test\":\"$test_filter\",\"error_context\":\"rch remote stall timeout\"},\"outcome\":\"failed\",\"reason_code\":\"rch_remote_stall\",\"error_code\":\"RCH-REMOTE-STALL\",\"artifact_path\":\"${queue_log#"$ROOT_DIR"/}\"}"
  log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"complete\",\"status\":\"failed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"suite\",\"inputs\":{\"error_context\":\"rch remote stall timeout\"},\"outcome\":\"failed\",\"reason_code\":\"rch_remote_stall\",\"error_code\":\"RCH-REMOTE-STALL\",\"artifact_path\":\"${json_log#"$ROOT_DIR"/}\"}"
  fatal "RCH-REMOTE-STALL: rch remote command timed out after ${RCH_STEP_TIMEOUT_SECS}s. See ${queue_log}"
fi

assert_nonzero_tests_ran "$raw_log"

if [[ $rc -eq 0 ]]; then

  log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"cargo_test\",\"status\":\"passed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"cargo_test\",\"inputs\":{\"test\":\"$test_filter\"},\"outcome\":\"pass\",\"reason_code\":null,\"error_code\":null,\"assertions\":[\"runtime emits egress replay capture events\",\"runtime emits lifecycle replay capture events\",\"captured events include deterministic event_id values\"],\"artifact_path\":\"${raw_log#"$ROOT_DIR"/}\"}"
  log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"complete\",\"status\":\"passed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"suite\",\"inputs\":{},\"outcome\":\"pass\",\"reason_code\":null,\"error_code\":null,\"artifact_path\":\"${json_log#"$ROOT_DIR"/}\"}"
  echo "Replay capture extraction e2e passed. Logs: ${json_log#"$ROOT_DIR"/}"
  exit 0
fi

log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"cargo_test\",\"status\":\"failed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"cargo_test\",\"inputs\":{\"test\":\"$test_filter\",\"error_context\":\"see cargo raw log\"},\"outcome\":\"failed\",\"reason_code\":\"cargo_test_failed\",\"error_code\":$rc,\"artifact_path\":\"${raw_log#"$ROOT_DIR"/}\"}"
log_json "{\"timestamp\":\"$(now_ts)\",\"component\":\"$component\",\"run_id\":\"$run_id\",\"scenario_id\":\"$scenario_id\",\"pane_id\":null,\"step\":\"complete\",\"status\":\"failed\",\"correlation_id\":\"$run_id\",\"decision_path\":\"suite\",\"inputs\":{\"error_context\":\"cargo test command failed\"},\"outcome\":\"failed\",\"reason_code\":\"cargo_test_failed\",\"error_code\":$rc,\"artifact_path\":\"${json_log#"$ROOT_DIR"/}\"}"

echo "Replay capture extraction e2e failed. Logs: ${json_log#"$ROOT_DIR"/}" >&2
tail -n 80 "$raw_log" >&2 || true
exit "$rc"
