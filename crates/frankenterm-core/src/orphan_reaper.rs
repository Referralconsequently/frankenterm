//! Orphan reaper for stuck `wezterm cli` helper processes.
//!
//! FrankenTerm spawns short-lived `wezterm cli <subcommand>` processes to query
//! and control the WezTerm mux backend.  These can hang due to lock contention,
//! socket timeouts, or notification feedback loops.  The orphan reaper
//! periodically scans for such processes and kills any that exceed a
//! configurable age threshold.
//!
//! # Proxy safety
//!
//! `wezterm cli --prefer-mux proxy` (and other `proxy` invocations) are
//! **long-lived SSH session transport processes**.  Killing them severs active
//! SSH sessions.  The reaper therefore maintains an explicit allowlist of
//! short-lived helper subcommands and **never** touches `proxy` or any
//! unrecognized subcommand.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tracing::{debug, info, warn};

use crate::config::CliConfig;

/// Short-lived `wezterm cli` subcommands that FrankenTerm spawns and that are
/// safe to reap when they exceed the age threshold.
///
/// This allowlist exists because `wezterm cli` also has long-lived subcommands
/// (notably `proxy`, used as SSH session transport via `--prefer-mux proxy`)
/// that must NEVER be killed.  Rather than trying to enumerate dangerous
/// subcommands (a fragile denylist), we enumerate only the ones we spawn.
const REAPABLE_SUBCOMMANDS: &[&str] = &[
    "list",
    "get-text",
    "send-text",
    "spawn",
    "split-pane",
    "activate-pane",
    "kill-pane",
    "zoom-pane",
    "list-clients",
    "get-pane-direction",
];

/// A process entry parsed from `ps` output.
#[derive(Debug)]
struct ProcessEntry {
    pid: u32,
    /// Elapsed time in seconds since the process started.
    age_seconds: u64,
    /// The full command line.
    command: String,
}

/// Run the orphan reaper loop.  Returns when `shutdown_flag` is set or the
/// reap interval is configured to zero (disabled).
pub async fn run_orphan_reaper(config: CliConfig, shutdown_flag: Arc<AtomicBool>) {
    let interval = config.orphan_reap_interval_seconds;
    if interval == 0 {
        info!("orphan reaper disabled (orphan_reap_interval_seconds = 0)");
        return;
    }

    let max_age = config.orphan_max_age_seconds;
    info!(
        interval_s = interval,
        max_age_s = max_age,
        "orphan reaper started"
    );

    loop {
        crate::runtime_compat::sleep(Duration::from_secs(interval)).await;

        if shutdown_flag.load(Ordering::Relaxed) {
            debug!("orphan reaper shutting down");
            return;
        }

        match scan_and_reap(max_age).await {
            Ok((scanned, killed)) => {
                if killed > 0 {
                    info!(scanned, killed, "orphan reaper cycle complete");
                } else {
                    debug!(scanned, "orphan reaper cycle — no orphans");
                }
            }
            Err(e) => {
                warn!(error = %e, "orphan reaper scan failed");
            }
        }
    }
}

/// Scan for orphaned `wezterm cli` processes and kill those exceeding
/// `max_age_seconds`.  Returns `(scanned, killed)`.
async fn scan_and_reap(max_age_seconds: u64) -> Result<(usize, usize), String> {
    let entries = list_wezterm_cli_processes_via_ps().await?;
    let scanned = entries.len();
    let mut killed = 0;

    for entry in &entries {
        if entry.age_seconds >= max_age_seconds {
            debug!(
                pid = entry.pid,
                age_s = entry.age_seconds,
                cmd = %entry.command,
                "killing orphaned wezterm cli process"
            );
            // Best-effort SIGKILL.  We intentionally ignore errors (the
            // process may have exited between the scan and the kill).
            // Use runtime_compat::spawn_blocking + std::process::Command
            // to avoid requiring a Tokio reactor (panics under asupersync).
            let pid_str = entry.pid.to_string();
            let _ = crate::runtime_compat::spawn_blocking(move || {
                std::process::Command::new("kill")
                    .args(["-s", "KILL", &pid_str])
                    .status()
            })
            .await;
            killed += 1;
        }
    }

    Ok((scanned, killed))
}

/// List `wezterm cli` processes that are candidates for reaping.
///
/// Uses `ps -eo pid,etimes,args` to get PID, elapsed time in seconds, and the
/// full command line.  Only processes whose command is a direct `wezterm cli
/// <subcommand>` invocation with a subcommand on the [`REAPABLE_SUBCOMMANDS`]
/// allowlist are returned.
///
/// Specifically excluded:
/// - `proxy` subcommand (long-lived SSH session transport — killing it severs
///   active sessions)
/// - Lines where `wezterm cli` appears only as an argument to another process
///   (e.g. `grep "wezterm cli"`, `zsh -c "wezterm cli list"`)
/// - Any unrecognized subcommand (defense in depth — only reap what we know)
async fn list_wezterm_cli_processes_via_ps() -> Result<Vec<ProcessEntry>, String> {
    // `etimes` gives elapsed time in seconds (POSIX, works on Linux and macOS).
    // Use runtime_compat::spawn_blocking + std::process::Command to avoid
    // requiring a Tokio reactor (panics under asupersync runtime).
    let output = crate::runtime_compat::spawn_blocking(|| {
        std::process::Command::new("ps")
            .args(["-eo", "pid,etimes,args"])
            .output()
    })
    .await
    .map_err(|e| format!("spawn_blocking failed: {e}"))?
    .map_err(|e| format!("failed to run ps: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "ps exited with status {}",
            output.status.code().unwrap_or(-1)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        // Skip the header line.
        if line.starts_with("PID") || line.is_empty() {
            continue;
        }

        if let Some(entry) = parse_ps_line_if_reapable(line) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// Parse a single `ps -eo pid,etimes,args` line and return a [`ProcessEntry`]
/// only if the command is a directly-invoked `wezterm cli <allowed-subcommand>`.
///
/// Returns `None` for:
/// - Non-wezterm processes
/// - Lines where `wezterm cli` appears only as an argument to a wrapper (grep,
///   shell -c, etc.)
/// - `wezterm cli proxy` and any other non-allowlisted subcommand
fn parse_ps_line_if_reapable(line: &str) -> Option<ProcessEntry> {
    // Expected format: "  PID  ELAPSED  ARGS..."
    // Fields are whitespace-separated, with ARGS potentially containing spaces.
    let mut parts = line.split_whitespace();
    let pid: u32 = parts.next()?.parse().ok()?;
    let age_seconds: u64 = parts.next()?.parse().ok()?;

    // Collect the remaining tokens as the argument vector.
    let tokens: Vec<&str> = parts.collect();
    if tokens.is_empty() {
        return None;
    }

    // The first token must be a wezterm binary (the last path component must
    // start with "wezterm").  This filters out wrapper processes like:
    //   grep "wezterm cli"
    //   zsh -c "wezterm cli list"
    //   bash /some/script.sh  (that happens to mention wezterm in later args)
    let binary = tokens[0];
    let binary_basename = binary.rsplit('/').next().unwrap_or(binary);
    if !binary_basename.starts_with("wezterm") {
        return None;
    }

    // Expect at least: wezterm cli <subcommand>
    // tokens[0] = "wezterm" (or "/path/to/wezterm")
    // tokens[1] = "cli"     (possibly after flags like --config-file)
    // tokens[N] = the subcommand

    // Find the "cli" token.  WezTerm allows global flags before "cli"
    // (e.g. `wezterm --config-file foo.toml cli list`), so we scan forward.
    let cli_pos = tokens.iter().position(|&t| t == "cli")?;

    // Find the subcommand: the first token after "cli" that does not start
    // with "-" (skip flags like `--prefer-mux` that appear between "cli" and
    // the subcommand).
    let subcommand = tokens[(cli_pos + 1)..]
        .iter()
        .find(|&&t| !t.starts_with('-'))?;

    // Only reap subcommands on the explicit allowlist.
    if !REAPABLE_SUBCOMMANDS.contains(subcommand) {
        return None;
    }

    let command = tokens.join(" ");

    Some(ProcessEntry {
        pid,
        age_seconds,
        command,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Positive cases: should be accepted --

    #[test]
    fn accepts_simple_list() {
        let line = "  1234   45 wezterm cli list";
        let entry = parse_ps_line_if_reapable(line).expect("should match");
        assert_eq!(entry.pid, 1234);
        assert_eq!(entry.age_seconds, 45);
        assert!(entry.command.contains("list"));
    }

    #[test]
    fn accepts_absolute_path() {
        let line = "  5678   90 /usr/bin/wezterm cli get-text --pane-id 3";
        let entry = parse_ps_line_if_reapable(line).expect("should match");
        assert_eq!(entry.pid, 5678);
        assert!(entry.command.contains("get-text"));
    }

    #[test]
    fn accepts_send_text() {
        let line = "  100   120 wezterm cli send-text --pane-id 1 hello world";
        assert!(parse_ps_line_if_reapable(line).is_some());
    }

    #[test]
    fn accepts_spawn() {
        let line = "  200   35 wezterm cli spawn --new-window";
        assert!(parse_ps_line_if_reapable(line).is_some());
    }

    #[test]
    fn accepts_split_pane() {
        let line = "  300   50 /opt/wezterm cli split-pane --right";
        assert!(parse_ps_line_if_reapable(line).is_some());
    }

    #[test]
    fn accepts_with_global_flags() {
        // Global flags before "cli"
        let line = "  400   60 wezterm --config-file /tmp/wez.toml cli list";
        let entry = parse_ps_line_if_reapable(line).expect("should match");
        assert!(entry.command.contains("list"));
    }

    #[test]
    fn accepts_with_prefer_mux_flag_before_list() {
        // --prefer-mux is a flag to "cli", subcommand is "list" => reapable
        let line = "  500   70 wezterm cli --prefer-mux list";
        let entry = parse_ps_line_if_reapable(line).expect("should match");
        assert!(entry.command.contains("list"));
    }

    #[test]
    fn accepts_all_allowlisted_subcommands() {
        for sub in REAPABLE_SUBCOMMANDS {
            let line = format!("  999   40 wezterm cli {sub}");
            assert!(
                parse_ps_line_if_reapable(&line).is_some(),
                "subcommand '{sub}' should be accepted"
            );
        }
    }

    // -- Negative cases: must NOT be accepted --

    #[test]
    fn rejects_proxy() {
        let line = "  1000   500 wezterm cli proxy";
        assert!(parse_ps_line_if_reapable(line).is_none());
    }

    #[test]
    fn rejects_prefer_mux_proxy() {
        let line = "  1001   600 wezterm cli --prefer-mux proxy";
        assert!(parse_ps_line_if_reapable(line).is_none());
    }

    #[test]
    fn rejects_grep_containing_wezterm_cli() {
        let line = r#"  2000   10 grep wezterm cli list"#;
        assert!(parse_ps_line_if_reapable(line).is_none());
    }

    #[test]
    fn rejects_shell_wrapper() {
        let line = r#"  2001   10 zsh -c wezterm cli list"#;
        assert!(parse_ps_line_if_reapable(line).is_none());
    }

    #[test]
    fn rejects_bash_wrapper() {
        let line = r#"  2002   10 bash -c wezterm cli list"#;
        assert!(parse_ps_line_if_reapable(line).is_none());
    }

    #[test]
    fn rejects_unknown_subcommand() {
        let line = "  3000   40 wezterm cli some-future-cmd";
        assert!(parse_ps_line_if_reapable(line).is_none());
    }

    #[test]
    fn rejects_non_wezterm_binary() {
        let line = "  4000   40 notwezterm cli list";
        assert!(parse_ps_line_if_reapable(line).is_none());
    }

    #[test]
    fn rejects_header_line() {
        let line = "  PID ELAPSED COMMAND";
        assert!(parse_ps_line_if_reapable(line).is_none());
    }

    #[test]
    fn rejects_empty_line() {
        assert!(parse_ps_line_if_reapable("").is_none());
    }

    #[test]
    fn rejects_wezterm_without_cli() {
        let line = "  5000   40 wezterm start --always-new-process";
        assert!(parse_ps_line_if_reapable(line).is_none());
    }

    #[test]
    fn rejects_wezterm_cli_without_subcommand() {
        let line = "  6000   40 wezterm cli";
        assert!(parse_ps_line_if_reapable(line).is_none());
    }

    #[test]
    fn rejects_wezterm_cli_with_only_flags() {
        let line = "  6001   40 wezterm cli --prefer-mux";
        assert!(parse_ps_line_if_reapable(line).is_none());
    }
}
