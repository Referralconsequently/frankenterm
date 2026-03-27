#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/tests/e2e/logs"
ARTIFACT_DIR_BASE="${ROOT_DIR}/tests/e2e/artifacts/swarm_stress"
RUN_ID="${RUN_ID:-$(date -u +"%Y%m%d_%H%M%S")}"
SCENARIO_ID="ft_1memj_30_swarm_stress"
CORRELATION_ID="ft-1memj.30-${RUN_ID}"
TARGET_DIR_REL="${TARGET_DIR_REL:-target/rch-e2e-ft-1memj-30-${RUN_ID}}"
LOG_FILE="${LOG_DIR}/${SCENARIO_ID}_${RUN_ID}.jsonl"
ARTIFACT_DIR="${ARTIFACT_DIR_BASE}/${RUN_ID}"

mkdir -p "${LOG_DIR}" "${ARTIFACT_DIR}"

source "${ROOT_DIR}/tests/e2e/lib_rch_guards.sh"
RCH_SKIP_SMOKE_PREFLIGHT="${RCH_SKIP_SMOKE_PREFLIGHT:-1}"
rch_init "${LOG_DIR}" "${RUN_ID}" "1memj_30_swarm_stress" "${ROOT_DIR}"
ensure_rch_ready

usage() {
  cat <<'EOF'
Usage: scripts/e2e_swarm_stress.sh

Environment:
  TARGET_DIR_REL   Repo-relative cargo target dir for rch offload
  RUN_ID           Override generated run id
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 3
      ;;
  esac
done

require_repo_relative_target_dir() {
  case "${TARGET_DIR_REL}" in
    /*|../*|*/../*|..)
      echo "TARGET_DIR_REL must stay under the repo root for rch offload: ${TARGET_DIR_REL}" >&2
      exit 2
      ;;
  esac
}

emit_event() {
  local outcome="$1"
  local decision_path="$2"
  local reason_code="$3"
  local error_code="$4"
  local artifact_path="$5"
  local details="$6"
  local ts

  ts="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

  jq -cn \
    --arg timestamp "${ts}" \
    --arg component "swarm_stress.e2e" \
    --arg scenario_id "${SCENARIO_ID}" \
    --arg correlation_id "${CORRELATION_ID}" \
    --arg decision_path "${decision_path}" \
    --arg outcome "${outcome}" \
    --arg reason_code "${reason_code}" \
    --arg error_code "${error_code}" \
    --arg artifact_path "${artifact_path}" \
    --arg details "${details}" \
    '{
      record_type: "suite_event",
      timestamp: $timestamp,
      component: $component,
      scenario_id: $scenario_id,
      correlation_id: $correlation_id,
      decision_path: $decision_path,
      outcome: $outcome,
      reason_code: $reason_code,
      error_code: $error_code,
      artifact_path: $artifact_path,
      details: $details
    }' >> "${LOG_FILE}"
}

append_metric() {
  local decision_path="$1"
  local artifact_path="$2"
  local metric_json="$3"
  local ts

  ts="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

  jq -cn \
    --arg timestamp "${ts}" \
    --arg component "swarm_stress.e2e" \
    --arg scenario_id "${SCENARIO_ID}" \
    --arg correlation_id "${CORRELATION_ID}" \
    --arg decision_path "${decision_path}" \
    --arg artifact_path "${artifact_path}" \
    --argjson metric "${metric_json}" \
    '$metric + {
      record_type: "swarm_metric",
      timestamp: $timestamp,
      component: $component,
      scenario_id: $scenario_id,
      correlation_id: $correlation_id,
      decision_path: $decision_path,
      artifact_path: $artifact_path
    }' >> "${LOG_FILE}"
}

run_swarm_case() {
  local metric_name="$1"
  local rust_test_name="$2"
  local decision_path="$3"
  local stdout_file="${ARTIFACT_DIR}/${metric_name}.stdout.log"
  local metrics_found=0

  emit_event \
    "running" \
    "${decision_path}" \
    "none" \
    "none" \
    "$(basename "${stdout_file}")" \
    "cargo test -p frankenterm-core --test e2e_swarm_stress_core ${rust_test_name} -- --exact --nocapture"

  if ! run_rch_cargo_logged \
    "${stdout_file}" \
    env CARGO_TARGET_DIR="${TARGET_DIR_REL}" \
    cargo test -p frankenterm-core --test e2e_swarm_stress_core "${rust_test_name}" -- --exact --nocapture; then
    emit_event \
      "failed" \
      "${decision_path}" \
      "cargo_test_failed" \
      "cargo_command_failed" \
      "$(basename "${stdout_file}")" \
      "stress case failed"
    return 1
  fi

  while IFS= read -r metric_line; do
    local metric_json="${metric_line#FT_SWARM_METRIC }"
    append_metric "${decision_path}" "$(basename "${stdout_file}")" "${metric_json}"
    metrics_found=1
  done < <(grep '^FT_SWARM_METRIC ' "${stdout_file}" || true)

  if [[ "${metrics_found}" -ne 1 ]]; then
    emit_event \
      "failed" \
      "${decision_path}" \
      "missing_metric_output" \
      "swarm_metric_missing" \
      "$(basename "${stdout_file}")" \
      "expected FT_SWARM_METRIC line in cargo test output"
    return 1
  fi

  emit_event \
    "passed" \
    "${decision_path}" \
    "stress_case_passed" \
    "none" \
    "$(basename "${stdout_file}")" \
    "metric=${metric_name}"
}

emit_summary() {
  local summary_json

  summary_json="$(jq -s '
    def tier_rank($tier):
      if $tier == "Green" then 0
      elif $tier == "Yellow" then 1
      elif $tier == "Red" then 2
      elif $tier == "Black" then 3
      else -1
      end;
    {
      tests_run: ([.[] | select(.record_type == "swarm_metric")] | length),
      peak_rss_mb: ([.[] | select(.record_type == "swarm_metric") | .rss_mb // empty] | max // null),
      max_duration_s: ([.[] | select(.record_type == "swarm_metric") | .duration_s // empty] | max // null),
      highest_backpressure_tier: (
        [ .[] | select(.record_type == "swarm_metric") | .backpressure_tier // empty ] as $tiers
        | if ($tiers | length) == 0
          then null
          else ($tiers | sort_by(tier_rank(.)) | last)
          end
      )
    }' "${LOG_FILE}")"

  jq -cn \
    --arg timestamp "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
    --arg component "swarm_stress.e2e" \
    --arg scenario_id "${SCENARIO_ID}" \
    --arg correlation_id "${CORRELATION_ID}" \
    --argjson summary "${summary_json}" \
    '{
      record_type: "suite_summary",
      timestamp: $timestamp,
      component: $component,
      scenario_id: $scenario_id,
      correlation_id: $correlation_id
    } + $summary' >> "${LOG_FILE}"
}

main() {
  require_repo_relative_target_dir

  emit_event \
    "started" \
    "suite_init" \
    "none" \
    "none" \
    "$(basename "${LOG_FILE}")" \
    "swarm stress suite init"

  run_swarm_case "stress_50_panes_idle" "scale_50_panes_idle_hot_only" "idle_50"
  run_swarm_case "stress_100_panes_idle" "scale_100_panes_idle_hot_only" "idle_100"
  run_swarm_case "stress_200_panes_idle" "scale_200_panes_idle_hot_only" "idle_200"
  run_swarm_case "stress_50_panes_active" "scale_50_panes_with_warm_tier" "active_50"
  run_swarm_case "stress_200_panes_active" "scale_200_panes_with_warm_tier" "active_200"
  run_swarm_case "stress_single_pane_10mb" "single_pane_100k_lines_throughput" "single_pane_10mb"
  run_swarm_case "stress_rapid_pane_create_destroy" "rapid_create_destroy_100_panes" "pane_churn"
  run_swarm_case "stress_200_panes_backpressure" "coordinator_emergency_at_200_panes" "backpressure_200"

  emit_summary

  emit_event \
    "passed" \
    "suite_complete" \
    "all_cases_passed" \
    "none" \
    "$(basename "${LOG_FILE}")" \
    "swarm stress suite complete"

  echo "Scenario: ${SCENARIO_ID}"
  echo "Logs: tests/e2e/logs/$(basename "${LOG_FILE}")"
}

main "$@"
