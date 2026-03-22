use crate::sshd::*;
use frankenterm_ssh::runtime::block_on;
use frankenterm_ssh::ExecResult;
use portable_pty::Child;
use rstest::*;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

fn read_all(mut reader: impl Read) -> String {
    let mut output = String::new();
    reader
        .read_to_string(&mut output)
        .expect("failed to read exec stream");
    output
}

fn collect_exec_result(result: ExecResult) -> (String, String, u32) {
    let ExecResult {
        stdout,
        stderr,
        mut child,
        ..
    } = result;

    let stdout = read_all(stdout);
    let stderr = read_all(stderr);
    let status = child.wait().expect("exec wait failed");

    (stdout, stderr, status.exit_code())
}

fn log_exec_outcome(scenario_id: &str, command: &str, stdout: &str, stderr: &str, exit_code: u32) {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis();
    let reason_code = if exit_code == 0 {
        "exec_ok"
    } else {
        "exec_non_zero_exit"
    };

    eprintln!(
        "{{\"timestamp_ms\":{timestamp_ms},\"component\":\"frankenterm_ssh_e2e_exec\",\"scenario_id\":\"{scenario_id}\",\"command\":{command:?},\"outcome\":\"exec_completed\",\"reason_code\":\"{reason_code}\",\"exit_code\":{exit_code},\"stdout\":{stdout:?},\"stderr\":{stderr:?}}}"
    );
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn exec_should_capture_stdout_stderr_and_zero_exit(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    block_on(async {
        let session: SessionWithSshd = session.await;
        let command = "sh -lc 'printf stdout; printf stderr >&2; exit 0'";
        let result = session
            .exec(command, None)
            .await
            .expect("exec should succeed");
        let (stdout, stderr, exit_code) = collect_exec_result(result);

        log_exec_outcome("exec-happy-path", command, &stdout, &stderr, exit_code);

        assert_eq!(stdout, "stdout");
        assert_eq!(stderr, "stderr");
        assert_eq!(exit_code, 0);
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn exec_should_recover_after_non_zero_exit_on_same_session(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    block_on(async {
        let session: SessionWithSshd = session.await;

        let failing_command = "sh -lc 'printf injected-failure >&2; exit 23'";
        let failing = session
            .exec(failing_command, None)
            .await
            .expect("failing exec should still establish the channel");
        let (stdout, stderr, exit_code) = collect_exec_result(failing);

        log_exec_outcome(
            "exec-failure-path",
            failing_command,
            &stdout,
            &stderr,
            exit_code,
        );

        assert!(stdout.is_empty(), "unexpected stdout from failing exec");
        assert_eq!(stderr, "injected-failure");
        assert_eq!(exit_code, 23);

        let recovery_command = "sh -lc 'printf recovered-output; exit 0'";
        let recovered = session
            .exec(recovery_command, None)
            .await
            .expect("session should recover after a non-zero exit");
        let (stdout, stderr, exit_code) = collect_exec_result(recovered);

        log_exec_outcome(
            "exec-recovery-path",
            recovery_command,
            &stdout,
            &stderr,
            exit_code,
        );

        assert_eq!(stdout, "recovered-output");
        assert!(stderr.is_empty(), "unexpected stderr from recovery exec");
        assert_eq!(exit_code, 0);
    })
}
