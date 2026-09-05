use super::*;

#[test]
fn parses_linux_summary_and_preserves_reply_on_unknown_format() {
    assert_eq!(
        parse_output(Some(0), b"rtt min/avg/max/mdev = 0.012/0.012/0.012/0.000 ms\n", b""),
        PingOutcome::Reply { latency_ms: Some(0.012) },
    );
    assert_eq!(parse_output(Some(0), b"unknown format", b""), PingOutcome::Reply { latency_ms: None });
    assert_eq!(parse_output(Some(1), b"", b""), PingOutcome::NoReply);
    assert!(matches!(parse_output(Some(2), b"", b"Name resolution failed"), PingOutcome::Error(_)));
}

#[cfg(unix)]
#[tokio::test]
async fn enforces_deadline_and_reaps_child() {
    let dir = tempfile::tempdir().unwrap();
    let pid_file = dir.path().join("pid");
    let mut command = Command::new("sh");
    command.args(["-c", "echo $$ > \"$1\"; exec sleep 30", "probe"]).arg(&pid_file);
    let outcome = execute(command, Duration::from_millis(500)).await;
    assert!(matches!(outcome, PingOutcome::Error(message) if message.contains("deadline")));
    let pid = std::fs::read_to_string(pid_file).unwrap();
    let status = std::process::Command::new("kill")
        .args(["-0", pid.trim()]).stderr(Stdio::null()).status().unwrap();
    assert!(!status.success(), "timed-out child should already be reaped");
}

#[cfg(unix)]
#[tokio::test]
async fn dropping_worker_kills_its_child() {
    let dir = tempfile::tempdir().unwrap();
    let pid_file = dir.path().join("pid");
    let path = pid_file.clone();
    let worker = tokio::spawn(async move {
        let mut command = Command::new("sh");
        command.args(["-c", "echo $$ > \"$1\"; exec sleep 30", "probe"]).arg(path);
        execute(command, Duration::from_secs(40)).await
    });
    for _ in 0..100 {
        if pid_file.exists() { break; }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let pid = std::fs::read_to_string(pid_file).unwrap();
    worker.abort();
    let _ = worker.await;
    for _ in 0..100 {
        let alive = std::process::Command::new("kill")
            .args(["-0", pid.trim()]).stderr(Stdio::null()).status().unwrap().success();
        if !alive { return; }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("cancelled worker left its child running");
}
