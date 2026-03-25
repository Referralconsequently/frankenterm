//! Operator-tunable constants for ft deployment configuration.
//!
//! This module collects hard-coded constants from across the codebase into a
//! single, serde-deserializable structure that operators can override via the
//! `[tuning]` section of `ft.toml`.
//!
//! # Design Principles
//!
//! 1. **Zero-config by default**: Every field has a `Default` impl that matches
//!    the original hard-coded constant exactly. An empty `[tuning]` section (or
//!    no section at all) produces identical behavior to the pre-migration code.
//!
//! 2. **Additive schema**: New fields can be added without breaking existing
//!    config files thanks to `#[serde(default)]` on every struct.
//!
//! 3. **Validated at load time**: The `validate()` method catches out-of-range
//!    values before the runtime starts, producing clear error messages.
//!
//! # Origin
//!
//! Created as part of the hardcoded-constants audit (epic ft-8gmtq, track T1).
//! Each field documents the source file and line of the original constant.

use serde::{Deserialize, Serialize};

// =============================================================================
// Top-level TuningConfig
// =============================================================================

/// Operator-tunable constants loaded from `ft.toml` `[tuning]` section.
///
/// All fields use `#[serde(default)]` so missing keys produce the original
/// hard-coded values. This struct is immutable after loading — hot-reload is
/// a future enhancement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TuningConfig {
    /// Runtime observation loop parameters.
    pub runtime: RuntimeTuning,
    /// Backpressure detection thresholds.
    pub backpressure: BackpressureTuning,
    /// Session snapshot maintenance timings.
    pub snapshot: SnapshotTuning,
    /// Ingest pipeline parameters.
    pub ingest: IngestTuning,
    /// Pattern detection engine limits.
    pub patterns: PatternsTuning,
    /// Policy engine and rate limiting.
    pub policy: PolicyTuning,
    /// Audit trail retention and limits.
    pub audit: AuditTuning,
    /// Web API server parameters.
    pub web: WebTuning,
    /// Workflow execution limits.
    pub workflows: WorkflowsTuning,
    /// Search and query limits.
    pub search: SearchTuning,
    /// Wire protocol (distributed mode) limits.
    pub wire_protocol: WireProtocolTuning,
    /// IPC (local Unix socket RPC) parameters.
    pub ipc: IpcTuning,
    /// WezTerm backend adapter timeouts.
    pub wezterm: WeztermTuning,
}

impl Default for TuningConfig {
    fn default() -> Self {
        Self {
            runtime: RuntimeTuning::default(),
            backpressure: BackpressureTuning::default(),
            snapshot: SnapshotTuning::default(),
            ingest: IngestTuning::default(),
            patterns: PatternsTuning::default(),
            policy: PolicyTuning::default(),
            audit: AuditTuning::default(),
            web: WebTuning::default(),
            workflows: WorkflowsTuning::default(),
            search: SearchTuning::default(),
            wire_protocol: WireProtocolTuning::default(),
            ipc: IpcTuning::default(),
            wezterm: WeztermTuning::default(),
        }
    }
}

// =============================================================================
// RuntimeTuning
// =============================================================================

/// Runtime observation loop parameters.
///
/// Controls output buffering, telemetry, resize watchdog, and storage lock
/// thresholds in `runtime.rs`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct RuntimeTuning {
    /// Output coalesce window (ms). Batches pane output bursts before processing.
    /// Source: runtime.rs NATIVE_OUTPUT_COALESCE_WINDOW_MS = 50
    pub output_coalesce_window_ms: u64,

    /// Max delay guard for output coalescing (ms). Prevents indefinite buffering.
    /// Source: runtime.rs NATIVE_OUTPUT_COALESCE_MAX_DELAY_MS = 200
    pub output_coalesce_max_delay_ms: u64,

    /// Max bytes in a coalesced output batch.
    /// Source: runtime.rs NATIVE_OUTPUT_COALESCE_MAX_BYTES = 256 * 1024
    pub output_coalesce_max_bytes: usize,

    /// Recent samples retained for lock/memory percentile telemetry.
    /// Source: runtime.rs TELEMETRY_PERCENTILE_WINDOW_CAPACITY = 1024
    pub telemetry_percentile_window: usize,

    /// Warning threshold (ms) for stalled resize transactions.
    /// Source: runtime.rs RESIZE_WATCHDOG_WARNING_THRESHOLD_MS = 2000
    pub resize_watchdog_warning_ms: u64,

    /// Critical threshold (ms) for stalled resize transactions.
    /// Source: runtime.rs RESIZE_WATCHDOG_CRITICAL_THRESHOLD_MS = 8000
    pub resize_watchdog_critical_ms: u64,

    /// Consecutive critical stalls before safe-mode recommendation.
    /// Source: runtime.rs RESIZE_WATCHDOG_CRITICAL_STALLED_LIMIT = 2
    pub resize_watchdog_stalled_limit: usize,

    /// Max stalled-transaction samples retained in watchdog payloads.
    /// Source: runtime.rs RESIZE_WATCHDOG_SAMPLE_LIMIT = 8
    pub resize_watchdog_sample_limit: usize,

    /// Storage lock wait warning threshold (ms). Warns when SQLite lock
    /// acquisition exceeds this duration.
    /// Source: runtime.rs STORAGE_LOCK_WAIT_WARN_MS = 15.0
    pub storage_lock_wait_warn_ms: f64,

    /// Storage lock hold warning threshold (ms). Warns when SQLite lock
    /// is held longer than this.
    /// Source: runtime.rs STORAGE_LOCK_HOLD_WARN_MS = 75.0
    pub storage_lock_hold_warn_ms: f64,

    /// Memory warning threshold (bytes) for retained pane cursor snapshots.
    /// Source: runtime.rs CURSOR_SNAPSHOT_MEMORY_WARN_BYTES = 64MB
    pub cursor_snapshot_memory_warn_bytes: u64,

    /// Max age (seconds) for agent state detection data. Detections older than
    /// this are considered stale and excluded from state inference.
    /// Source: agent_correlator.rs + snapshot_engine.rs STATE_DETECTION_MAX_AGE = 300
    pub state_detection_max_age_secs: u64,
}

impl Default for RuntimeTuning {
    fn default() -> Self {
        Self {
            output_coalesce_window_ms: 50,
            output_coalesce_max_delay_ms: 200,
            output_coalesce_max_bytes: 256 * 1024,
            telemetry_percentile_window: 1024,
            resize_watchdog_warning_ms: 2_000,
            resize_watchdog_critical_ms: 8_000,
            resize_watchdog_stalled_limit: 2,
            resize_watchdog_sample_limit: 8,
            storage_lock_wait_warn_ms: 15.0,
            storage_lock_hold_warn_ms: 75.0,
            cursor_snapshot_memory_warn_bytes: 64 * 1024 * 1024,
            state_detection_max_age_secs: 300,
        }
    }
}

// =============================================================================
// BackpressureTuning
// =============================================================================

/// Backpressure detection thresholds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BackpressureTuning {
    /// Queue fullness ratio that triggers a warning in health snapshots.
    /// 0.75 = warn at 75% full.
    /// Source: runtime.rs BACKPRESSURE_WARN_RATIO = 0.75
    pub warn_ratio: f64,
}

impl Default for BackpressureTuning {
    fn default() -> Self {
        Self { warn_ratio: 0.75 }
    }
}

// =============================================================================
// SnapshotTuning
// =============================================================================

/// Session snapshot maintenance timings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SnapshotTuning {
    /// Interval (seconds) for snapshot trigger bridge maintenance checks.
    /// Source: runtime.rs SNAPSHOT_TRIGGER_BRIDGE_TICK_SECS = 30
    pub trigger_bridge_tick_secs: u64,

    /// Idle duration (seconds) before emitting `IdleWindow` trigger.
    /// Source: runtime.rs SNAPSHOT_IDLE_WINDOW_SECS = 300
    pub idle_window_secs: u64,

    /// Minimum interval (seconds) between memory-pressure-triggered snapshots.
    /// Source: runtime.rs SNAPSHOT_MEMORY_TRIGGER_COOLDOWN_SECS = 120
    pub memory_trigger_cooldown_secs: u64,
}

impl Default for SnapshotTuning {
    fn default() -> Self {
        Self {
            trigger_bridge_tick_secs: 30,
            idle_window_secs: 300,
            memory_trigger_cooldown_secs: 120,
        }
    }
}

// =============================================================================
// IngestTuning
// =============================================================================

/// Ingest pipeline parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct IngestTuning {
    /// Max bytes per persisted output segment. Controls SQLite write granularity.
    /// Source: ingest.rs DEFAULT_MAX_PERSIST_SEGMENT_BYTES = 64KB
    pub max_persist_segment_bytes: usize,

    /// Max single record payload for tantivy ingestion (bytes).
    /// Source: tantivy_ingest.rs MAX_RECORD_PAYLOAD = 64MB
    pub max_record_payload_bytes: usize,
}

impl Default for IngestTuning {
    fn default() -> Self {
        Self {
            max_persist_segment_bytes: 64 * 1024,
            max_record_payload_bytes: 64 * 1024 * 1024,
        }
    }
}

// =============================================================================
// PatternsTuning
// =============================================================================

/// Pattern detection engine limits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PatternsTuning {
    /// Max entries in the pattern dedup cache. Each unique
    /// (rule_id, pane_id, fingerprint) tuple occupies one slot.
    /// Source: patterns.rs MAX_SEEN_KEYS = 1000
    pub max_seen_keys: usize,

    /// Tail buffer size (bytes) for anchor matching. Retains the last N bytes
    /// of each pane's output for pattern context.
    /// Source: patterns.rs MAX_TAIL_SIZE = 2048
    pub max_tail_size_bytes: usize,

    /// Bloom filter false-positive rate for pattern pre-filtering.
    /// Lower = more memory, fewer false regex evaluations.
    /// Source: patterns.rs BLOOM_FALSE_POSITIVE_RATE = 0.01
    pub bloom_false_positive_rate: f64,
}

impl Default for PatternsTuning {
    fn default() -> Self {
        Self {
            max_seen_keys: 1000,
            max_tail_size_bytes: 2048,
            bloom_false_positive_rate: 0.01,
        }
    }
}

// =============================================================================
// PolicyTuning
// =============================================================================

/// Policy engine and rate limiting parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PolicyTuning {
    /// Sliding window (seconds) for send_text rate limiting.
    /// Source: policy.rs RATE_LIMIT_WINDOW = 60
    pub rate_limit_window_secs: u64,

    /// Max panes tracked by the rate limiter. When exceeded, oldest pane data
    /// is evicted (causing rate limit amnesia for that pane).
    /// Source: rate_limit_tracker.rs MAX_TRACKED_PANES = 256
    pub max_tracked_panes: usize,

    /// Max events retained per pane in the rate limiter.
    /// Source: rate_limit_tracker.rs MAX_EVENTS_PER_PANE = 64
    pub max_events_per_pane: usize,

    /// Max panes tracked by the cost tracker.
    /// Source: cost_tracker.rs MAX_TRACKED_PANES = 512
    pub cost_tracker_max_panes: usize,
}

impl Default for PolicyTuning {
    fn default() -> Self {
        Self {
            rate_limit_window_secs: 60,
            max_tracked_panes: 256,
            max_events_per_pane: 64,
            cost_tracker_max_panes: 512,
        }
    }
}

// =============================================================================
// AuditTuning
// =============================================================================

/// Audit trail retention and limits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AuditTuning {
    /// Days to retain audit log entries.
    /// Source: recorder_audit.rs DEFAULT_AUDIT_RETENTION_DAYS = 90
    pub retention_days: u32,

    /// Approval token time-to-live (seconds). After this, one-time approvals expire.
    /// Source: recorder_audit.rs DEFAULT_APPROVAL_TTL_SECONDS = 900
    pub approval_ttl_secs: u64,

    /// Max rows returned by raw audit queries.
    /// Source: recorder_audit.rs DEFAULT_MAX_RAW_QUERY_ROWS = 100
    pub max_raw_query_rows: usize,

    /// Days to retain replay artifacts.
    /// Source: replay_artifact_registry.rs DEFAULT_RETENTION_DAYS = 30
    pub artifact_retention_days: u32,

    /// Duration (days) for shadow rollout evaluation.
    /// Source: replay_shadow_rollout.rs DEFAULT_SHADOW_DAYS = 14
    pub shadow_rollout_days: u32,
}

impl Default for AuditTuning {
    fn default() -> Self {
        Self {
            retention_days: 90,
            approval_ttl_secs: 900,
            max_raw_query_rows: 100,
            artifact_retention_days: 30,
            shadow_rollout_days: 14,
        }
    }
}

// =============================================================================
// WebTuning
// =============================================================================

/// Web API server parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WebTuning {
    /// Bind address for the web server.
    /// Source: web.rs DEFAULT_HOST = "127.0.0.1"
    pub default_host: String,

    /// Port for the web server.
    /// Source: web.rs DEFAULT_PORT = 8000
    pub default_port: u16,

    /// Hard ceiling on list endpoint results.
    /// Source: web.rs MAX_LIMIT = 500
    pub max_list_limit: usize,

    /// Default page size for list endpoints.
    /// Source: web.rs DEFAULT_LIMIT = 50
    pub default_list_limit: usize,

    /// Max HTTP request body size (bytes).
    /// Source: web.rs MAX_REQUEST_BODY_BYTES = 64KB
    pub max_request_body_bytes: usize,

    /// Default max SSE event rate (Hz).
    /// Source: web.rs STREAM_DEFAULT_MAX_HZ = 50
    pub stream_default_max_hz: u32,

    /// Hard ceiling on SSE event rate (Hz).
    /// Source: web.rs STREAM_MAX_MAX_HZ = 500
    pub stream_max_max_hz: u32,

    /// SSE keep-alive ping interval (seconds).
    /// Source: web.rs STREAM_KEEPALIVE_SECS = 15
    pub stream_keepalive_secs: u64,

    /// Scan page size for streaming queries.
    /// Source: web.rs STREAM_SCAN_LIMIT = 256
    pub stream_scan_limit: usize,

    /// Max pages scanned per streaming query.
    /// Source: web.rs STREAM_SCAN_MAX_PAGES = 8
    pub stream_scan_max_pages: usize,
}

impl Default for WebTuning {
    fn default() -> Self {
        Self {
            default_host: "127.0.0.1".to_string(),
            default_port: 8000,
            max_list_limit: 500,
            default_list_limit: 50,
            max_request_body_bytes: 64 * 1024,
            stream_default_max_hz: 50,
            stream_max_max_hz: 500,
            stream_keepalive_secs: 15,
            stream_scan_limit: 256,
            stream_scan_max_pages: 8,
        }
    }
}

// =============================================================================
// WorkflowsTuning
// =============================================================================

/// Configuration for a single CASS (Cross-Agent Session Search) query handler.
///
/// Three handlers use this pattern with slightly different defaults:
/// session_start, on_error, and auth. This struct unifies the duplicated
/// constant blocks from workflows/handlers.rs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CassQueryConfig {
    /// Max hints to include in context injection.
    pub hint_limit: usize,
    /// CASS query timeout (seconds).
    pub timeout_secs: u64,
    /// How far back (days) to search session history.
    pub lookback_days: u32,
    /// Max characters in the query string sent to CASS.
    pub query_max_chars: usize,
    /// Max characters per hint in the response.
    pub hint_max_chars: usize,
}

impl CassQueryConfig {
    /// Defaults for the session-start handler.
    /// Source: workflows/handlers.rs lines 1015-1020
    pub fn session_start() -> Self {
        Self {
            hint_limit: 3,
            timeout_secs: 8,
            lookback_days: 30,
            query_max_chars: 180,
            hint_max_chars: 160,
        }
    }

    /// Defaults for the on-error handler.
    /// Source: workflows/handlers.rs lines 1572-1576
    /// Note: timeout is 6s (not 8s) — intentionally faster for error paths.
    pub fn on_error() -> Self {
        Self {
            hint_limit: 3,
            timeout_secs: 6,
            lookback_days: 30,
            query_max_chars: 200,
            hint_max_chars: 180,
        }
    }

    /// Defaults for the auth handler.
    /// Source: workflows/handlers.rs lines 2301-2305
    pub fn auth() -> Self {
        Self {
            hint_limit: 3,
            timeout_secs: 8,
            lookback_days: 30,
            query_max_chars: 160,
            hint_max_chars: 140,
        }
    }
}

impl Default for CassQueryConfig {
    fn default() -> Self {
        Self::session_start()
    }
}

/// Workflow execution limits and CASS handler configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WorkflowsTuning {
    /// Max steps per workflow descriptor.
    /// Source: workflows/descriptors.rs DESCRIPTOR_MAX_STEPS = 32
    pub max_steps: usize,

    /// Max wait-for timeout per step (ms).
    /// Source: workflows/descriptors.rs DESCRIPTOR_MAX_WAIT_TIMEOUT_MS = 120000
    pub max_wait_timeout_ms: u64,

    /// Max sleep duration per step (ms).
    /// Source: workflows/descriptors.rs DESCRIPTOR_MAX_SLEEP_MS = 30000
    pub max_sleep_ms: u64,

    /// Max text payload per step (bytes).
    /// Source: workflows/descriptors.rs DESCRIPTOR_MAX_TEXT_LEN = 8192
    pub max_text_len: usize,

    /// Max match pattern length (bytes).
    /// Source: workflows/descriptors.rs DESCRIPTOR_MAX_MATCH_LEN = 1024
    pub max_match_len: usize,

    /// CASS config for session-start handler.
    pub cass_session_start: CassQueryConfig,

    /// CASS config for on-error handler.
    pub cass_on_error: CassQueryConfig,

    /// CASS config for auth handler.
    pub cass_auth: CassQueryConfig,

    /// Swarm learning index timeout (seconds).
    /// Source: workflows/handlers.rs SWARM_LEARNING_INDEX_TIMEOUT_SECS = 30
    pub swarm_learning_index_timeout_secs: u64,

    /// Claude Code rate limit cooldown (ms).
    /// Source: workflows/handlers.rs CLAUDE_CODE_LIMITS_COOLDOWN_MS = 600000
    pub claude_code_limits_cooldown_ms: u64,

    /// Context injection cooldown (ms) for session-start handler.
    /// Source: workflows/handlers.rs SESSION_START_CONTEXT_COOLDOWN_MS = 600000
    pub session_start_context_cooldown_ms: u64,
}

impl Default for WorkflowsTuning {
    fn default() -> Self {
        Self {
            max_steps: 32,
            max_wait_timeout_ms: 120_000,
            max_sleep_ms: 30_000,
            max_text_len: 8192,
            max_match_len: 1024,
            cass_session_start: CassQueryConfig::session_start(),
            cass_on_error: CassQueryConfig::on_error(),
            cass_auth: CassQueryConfig::auth(),
            swarm_learning_index_timeout_secs: 30,
            claude_code_limits_cooldown_ms: 600_000,
            session_start_context_cooldown_ms: 600_000,
        }
    }
}

// =============================================================================
// SearchTuning
// =============================================================================

/// Search and query parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SearchTuning {
    /// Default result limit for search queries.
    /// Source: query_contract.rs SEARCH_LIMIT_DEFAULT = 20
    pub default_limit: usize,

    /// Hard ceiling on search result count.
    /// Source: query_contract.rs SEARCH_LIMIT_MAX = 1000
    pub max_limit: usize,

    /// Default limit for saved search list queries.
    /// Source: storage.rs SAVED_SEARCH_DEFAULT_LIMIT = 50
    pub saved_search_limit: usize,

    /// Max records exported via CASS export.
    /// Source: cass.rs DEFAULT_CASS_EXPORT_LIMIT = 1000
    pub cass_export_limit: usize,

    /// Tantivy index writer memory budget (bytes).
    /// Source: recorder_lexical_ingest.rs DEFAULT_WRITER_MEMORY_BYTES = 50MB
    pub tantivy_writer_memory_bytes: usize,
}

impl Default for SearchTuning {
    fn default() -> Self {
        Self {
            default_limit: 20,
            max_limit: 1000,
            saved_search_limit: 50,
            cass_export_limit: 1000,
            tantivy_writer_memory_bytes: 50 * 1024 * 1024,
        }
    }
}

// =============================================================================
// WireProtocolTuning
// =============================================================================

/// Wire protocol (distributed mode) limits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WireProtocolTuning {
    /// Max message size for the distributed wire protocol (bytes).
    /// Both sender and receiver must agree on this value.
    /// Source: wire_protocol.rs MAX_MESSAGE_SIZE = 1MB
    pub max_message_size: usize,

    /// Max sender ID length (bytes).
    /// Source: wire_protocol.rs MAX_SENDER_ID_LEN = 128
    pub max_sender_id_len: usize,
}

impl Default for WireProtocolTuning {
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024,
            max_sender_id_len: 128,
        }
    }
}

// =============================================================================
// IpcTuning
// =============================================================================

/// IPC (local Unix socket RPC) parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct IpcTuning {
    /// Max IPC message size (bytes).
    /// Source: ipc.rs MAX_MESSAGE_SIZE = 131072
    pub max_message_size: usize,

    /// IPC connection accept poll interval (ms).
    /// Source: ipc.rs IPC_ACCEPT_POLL_INTERVAL = 100
    pub accept_poll_interval_ms: u64,
}

impl Default for IpcTuning {
    fn default() -> Self {
        Self {
            max_message_size: 128 * 1024,
            accept_poll_interval_ms: 100,
        }
    }
}

// =============================================================================
// WeztermTuning
// =============================================================================

/// WezTerm backend adapter timeouts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WeztermTuning {
    /// CLI command timeout (seconds).
    /// Source: wezterm.rs DEFAULT_TIMEOUT_SECS = 30
    pub timeout_secs: u64,

    /// Retry delay between CLI attempts (ms).
    /// Source: wezterm.rs DEFAULT_RETRY_DELAY_MS = 200
    pub retry_delay_ms: u64,

    /// Max error output retained from CLI (bytes).
    /// Source: wezterm.rs MAX_ERROR_BYTES = 8192
    pub max_error_bytes: usize,

    /// Mux socket connect timeout (ms).
    /// Source: vendored mux_client.rs DEFAULT_CONNECT_TIMEOUT_MS = 5000
    pub connect_timeout_ms: u64,

    /// Mux socket read timeout (ms).
    /// Source: vendored mux_client.rs DEFAULT_READ_TIMEOUT_MS = 5000
    pub read_timeout_ms: u64,

    /// Mux socket write timeout (ms).
    /// Source: vendored mux_client.rs DEFAULT_WRITE_TIMEOUT_MS = 5000
    pub write_timeout_ms: u64,
}

impl Default for WeztermTuning {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            retry_delay_ms: 200,
            max_error_bytes: 8192,
            connect_timeout_ms: 5000,
            read_timeout_ms: 5000,
            write_timeout_ms: 5000,
        }
    }
}

// =============================================================================
// Validation
// =============================================================================

impl TuningConfig {
    /// Validate all fields are within acceptable ranges.
    ///
    /// Returns a list of validation errors. Empty list means valid.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Runtime
        if self.runtime.output_coalesce_window_ms < 5 {
            errors.push("tuning.runtime.output_coalesce_window_ms must be >= 5".into());
        }
        if self.runtime.output_coalesce_max_delay_ms < self.runtime.output_coalesce_window_ms {
            errors.push(
                "tuning.runtime.output_coalesce_max_delay_ms must be >= output_coalesce_window_ms"
                    .into(),
            );
        }
        if self.runtime.output_coalesce_max_bytes < 4096 {
            errors.push("tuning.runtime.output_coalesce_max_bytes must be >= 4096".into());
        }
        if self.runtime.resize_watchdog_warning_ms >= self.runtime.resize_watchdog_critical_ms {
            errors.push(
                "tuning.runtime.resize_watchdog_warning_ms must be < resize_watchdog_critical_ms"
                    .into(),
            );
        }
        if self.runtime.resize_watchdog_stalled_limit < 1 {
            errors.push("tuning.runtime.resize_watchdog_stalled_limit must be >= 1".into());
        }

        // Backpressure
        if !(0.1..=0.99).contains(&self.backpressure.warn_ratio) {
            errors.push("tuning.backpressure.warn_ratio must be in [0.1, 0.99]".into());
        }

        // Snapshot
        if self.snapshot.trigger_bridge_tick_secs < 5 {
            errors.push("tuning.snapshot.trigger_bridge_tick_secs must be >= 5".into());
        }

        // Patterns
        if self.patterns.max_seen_keys < 100 {
            errors.push("tuning.patterns.max_seen_keys must be >= 100".into());
        }
        if self.patterns.max_tail_size_bytes < 256 {
            errors.push("tuning.patterns.max_tail_size_bytes must be >= 256".into());
        }
        if !(0.001..=0.2).contains(&self.patterns.bloom_false_positive_rate) {
            errors.push("tuning.patterns.bloom_false_positive_rate must be in [0.001, 0.2]".into());
        }

        // Policy
        if self.policy.rate_limit_window_secs < 10 {
            errors.push("tuning.policy.rate_limit_window_secs must be >= 10".into());
        }
        if self.policy.max_tracked_panes < 32 {
            errors.push("tuning.policy.max_tracked_panes must be >= 32".into());
        }
        if self.policy.max_events_per_pane < 8 {
            errors.push("tuning.policy.max_events_per_pane must be >= 8".into());
        }

        // Audit
        if self.audit.retention_days < 1 {
            errors.push("tuning.audit.retention_days must be >= 1".into());
        }
        if self.audit.approval_ttl_secs < 60 {
            errors.push("tuning.audit.approval_ttl_secs must be >= 60".into());
        }

        // Web
        if self.web.default_list_limit > self.web.max_list_limit {
            errors
                .push("tuning.web.default_list_limit must be <= tuning.web.max_list_limit".into());
        }
        if self.web.max_list_limit < 10 {
            errors.push("tuning.web.max_list_limit must be >= 10".into());
        }
        if self.web.stream_keepalive_secs < 1 {
            errors.push("tuning.web.stream_keepalive_secs must be >= 1".into());
        }

        // Workflows
        if self.workflows.max_steps < 4 {
            errors.push("tuning.workflows.max_steps must be >= 4".into());
        }
        if self.workflows.max_wait_timeout_ms < 1000 {
            errors.push("tuning.workflows.max_wait_timeout_ms must be >= 1000".into());
        }

        // Search
        if self.search.default_limit > self.search.max_limit {
            errors.push("tuning.search.default_limit must be <= tuning.search.max_limit".into());
        }
        if self.search.max_limit < 10 {
            errors.push("tuning.search.max_limit must be >= 10".into());
        }
        if self.search.tantivy_writer_memory_bytes < 10 * 1024 * 1024 {
            errors.push("tuning.search.tantivy_writer_memory_bytes must be >= 10MB".into());
        }

        // Wire protocol
        if self.wire_protocol.max_message_size < 64 * 1024 {
            errors.push("tuning.wire_protocol.max_message_size must be >= 64KB".into());
        }
        if self.wire_protocol.max_message_size > 64 * 1024 * 1024 {
            errors.push("tuning.wire_protocol.max_message_size must be <= 64MB".into());
        }

        // IPC
        if self.ipc.max_message_size < 16 * 1024 {
            errors.push("tuning.ipc.max_message_size must be >= 16KB".into());
        }
        if self.ipc.accept_poll_interval_ms < 10 {
            errors.push("tuning.ipc.accept_poll_interval_ms must be >= 10".into());
        }

        errors
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_validates_clean() {
        let cfg = TuningConfig::default();
        let errors = cfg.validate();
        assert!(
            errors.is_empty(),
            "default config should validate: {errors:?}"
        );
    }

    #[test]
    fn serde_roundtrip_preserves_defaults() {
        let original = TuningConfig::default();
        let toml_str = toml::to_string_pretty(&original).expect("serialize");
        let deserialized: TuningConfig = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(original, deserialized);
    }

    #[test]
    fn partial_override_preserves_other_defaults() {
        let toml_str = r#"
[runtime]
output_coalesce_window_ms = 100

[web]
default_port = 9000
"#;
        let cfg: TuningConfig = toml::from_str(toml_str).expect("parse");
        assert_eq!(cfg.runtime.output_coalesce_window_ms, 100);
        assert_eq!(cfg.web.default_port, 9000);
        // Other fields retain defaults
        assert_eq!(cfg.runtime.output_coalesce_max_delay_ms, 200);
        assert_eq!(cfg.backpressure.warn_ratio, 0.75);
        assert_eq!(cfg.policy.rate_limit_window_secs, 60);
    }

    #[test]
    fn defaults_match_current_hardcoded_constants() {
        let cfg = TuningConfig::default();

        // Runtime (runtime.rs)
        assert_eq!(cfg.runtime.output_coalesce_window_ms, 50);
        assert_eq!(cfg.runtime.output_coalesce_max_delay_ms, 200);
        assert_eq!(cfg.runtime.output_coalesce_max_bytes, 256 * 1024);
        assert_eq!(cfg.runtime.telemetry_percentile_window, 1024);
        assert_eq!(cfg.runtime.resize_watchdog_warning_ms, 2_000);
        assert_eq!(cfg.runtime.resize_watchdog_critical_ms, 8_000);
        assert_eq!(cfg.runtime.resize_watchdog_stalled_limit, 2);
        assert_eq!(cfg.runtime.resize_watchdog_sample_limit, 8);
        assert_eq!(cfg.runtime.storage_lock_wait_warn_ms, 15.0);
        assert_eq!(cfg.runtime.storage_lock_hold_warn_ms, 75.0);
        assert_eq!(
            cfg.runtime.cursor_snapshot_memory_warn_bytes,
            64 * 1024 * 1024
        );
        assert_eq!(cfg.runtime.state_detection_max_age_secs, 300);

        // Backpressure
        assert_eq!(cfg.backpressure.warn_ratio, 0.75);

        // Snapshot
        assert_eq!(cfg.snapshot.trigger_bridge_tick_secs, 30);
        assert_eq!(cfg.snapshot.idle_window_secs, 300);
        assert_eq!(cfg.snapshot.memory_trigger_cooldown_secs, 120);

        // Ingest
        assert_eq!(cfg.ingest.max_persist_segment_bytes, 64 * 1024);
        assert_eq!(cfg.ingest.max_record_payload_bytes, 64 * 1024 * 1024);

        // Patterns
        assert_eq!(cfg.patterns.max_seen_keys, 1000);
        assert_eq!(cfg.patterns.max_tail_size_bytes, 2048);
        assert_eq!(cfg.patterns.bloom_false_positive_rate, 0.01);

        // Policy
        assert_eq!(cfg.policy.rate_limit_window_secs, 60);
        assert_eq!(cfg.policy.max_tracked_panes, 256);
        assert_eq!(cfg.policy.max_events_per_pane, 64);
        assert_eq!(cfg.policy.cost_tracker_max_panes, 512);

        // Audit
        assert_eq!(cfg.audit.retention_days, 90);
        assert_eq!(cfg.audit.approval_ttl_secs, 900);
        assert_eq!(cfg.audit.max_raw_query_rows, 100);
        assert_eq!(cfg.audit.artifact_retention_days, 30);
        assert_eq!(cfg.audit.shadow_rollout_days, 14);

        // Web
        assert_eq!(cfg.web.default_host, "127.0.0.1");
        assert_eq!(cfg.web.default_port, 8000);
        assert_eq!(cfg.web.max_list_limit, 500);
        assert_eq!(cfg.web.default_list_limit, 50);
        assert_eq!(cfg.web.max_request_body_bytes, 64 * 1024);
        assert_eq!(cfg.web.stream_default_max_hz, 50);
        assert_eq!(cfg.web.stream_max_max_hz, 500);
        assert_eq!(cfg.web.stream_keepalive_secs, 15);
        assert_eq!(cfg.web.stream_scan_limit, 256);
        assert_eq!(cfg.web.stream_scan_max_pages, 8);

        // Workflows
        assert_eq!(cfg.workflows.max_steps, 32);
        assert_eq!(cfg.workflows.max_wait_timeout_ms, 120_000);
        assert_eq!(cfg.workflows.max_sleep_ms, 30_000);
        assert_eq!(cfg.workflows.max_text_len, 8192);
        assert_eq!(cfg.workflows.max_match_len, 1024);
        assert_eq!(cfg.workflows.swarm_learning_index_timeout_secs, 30);
        assert_eq!(cfg.workflows.claude_code_limits_cooldown_ms, 600_000);
        assert_eq!(cfg.workflows.session_start_context_cooldown_ms, 600_000);

        // CASS handler configs
        let ss = &cfg.workflows.cass_session_start;
        assert_eq!(ss.hint_limit, 3);
        assert_eq!(ss.timeout_secs, 8);
        assert_eq!(ss.lookback_days, 30);
        assert_eq!(ss.query_max_chars, 180);
        assert_eq!(ss.hint_max_chars, 160);

        let oe = &cfg.workflows.cass_on_error;
        assert_eq!(oe.timeout_secs, 6); // intentionally different
        assert_eq!(oe.query_max_chars, 200);
        assert_eq!(oe.hint_max_chars, 180);

        let au = &cfg.workflows.cass_auth;
        assert_eq!(au.timeout_secs, 8);
        assert_eq!(au.query_max_chars, 160);
        assert_eq!(au.hint_max_chars, 140);

        // Search
        assert_eq!(cfg.search.default_limit, 20);
        assert_eq!(cfg.search.max_limit, 1000);
        assert_eq!(cfg.search.saved_search_limit, 50);
        assert_eq!(cfg.search.cass_export_limit, 1000);
        assert_eq!(cfg.search.tantivy_writer_memory_bytes, 50 * 1024 * 1024);

        // Wire protocol
        assert_eq!(cfg.wire_protocol.max_message_size, 1024 * 1024);
        assert_eq!(cfg.wire_protocol.max_sender_id_len, 128);

        // IPC
        assert_eq!(cfg.ipc.max_message_size, 128 * 1024);
        assert_eq!(cfg.ipc.accept_poll_interval_ms, 100);

        // Wezterm
        assert_eq!(cfg.wezterm.timeout_secs, 30);
        assert_eq!(cfg.wezterm.retry_delay_ms, 200);
        assert_eq!(cfg.wezterm.max_error_bytes, 8192);
        assert_eq!(cfg.wezterm.connect_timeout_ms, 5000);
        assert_eq!(cfg.wezterm.read_timeout_ms, 5000);
        assert_eq!(cfg.wezterm.write_timeout_ms, 5000);
    }

    #[test]
    fn validation_catches_bad_values() {
        let mut cfg = TuningConfig::default();
        cfg.backpressure.warn_ratio = 1.5;
        cfg.patterns.bloom_false_positive_rate = 0.0;
        cfg.web.default_list_limit = 1000;
        cfg.web.max_list_limit = 5;

        let errors = cfg.validate();
        assert!(errors.len() >= 3, "expected multiple errors: {errors:?}");
    }

    #[test]
    fn cass_query_config_constructors_differ() {
        let ss = CassQueryConfig::session_start();
        let oe = CassQueryConfig::on_error();
        let au = CassQueryConfig::auth();

        assert_ne!(ss.timeout_secs, oe.timeout_secs);
        assert_ne!(ss.query_max_chars, oe.query_max_chars);
        assert_ne!(ss.hint_max_chars, au.hint_max_chars);
    }

    #[test]
    fn empty_toml_produces_default() {
        let cfg: TuningConfig = toml::from_str("").expect("parse empty");
        assert_eq!(cfg, TuningConfig::default());
    }
}
