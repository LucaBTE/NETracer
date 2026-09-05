use std::{process::Stdio, time::Duration};

use tokio::{io::AsyncReadExt, process::Command, time::timeout};

use super::target::Target;

#[derive(Clone, Debug, PartialEq)]
pub enum PingOutcome {
    Reply { latency_ms: Option<f64> },
    NoReply,
    Error(String),
}

pub async fn ping(target: &Target) -> PingOutcome {
    if !cfg!(target_os = "linux") {
        return PingOutcome::Error("The ping backend currently requires Linux iputils.".into());
    }

    let mut command = Command::new("ping");

    command
        .env("LC_ALL", "C")
        .args(["-n", "-c", "1", "-W", "1", "-w", "2", target.as_str()]);

    execute(command, Duration::from_secs(5)).await
}

async fn execute(mut command: Command, deadline: Duration) -> PingOutcome {
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn();

    let mut child = match child {
        Ok(child) => child,

        Err(error) => {
            return PingOutcome::Error(format!("Cannot start ping: {error}",));
        }
    };

    let mut stdout = child.stdout.take().expect("piped stdout");
    let mut stderr = child.stderr.take().expect("piped stderr");

    let mut output = Vec::new();
    let mut errors = Vec::new();

    // Read both pipes while waiting for the process.
    let result = timeout(deadline, async {
        tokio::try_join!(
            child.wait(),
            stdout.read_to_end(&mut output),
            stderr.read_to_end(&mut errors),
        )
    })
    .await;

    match result {
        Ok(Ok((status, _, _))) => parse_output(status.code(), &output, &errors),

        Ok(Err(error)) => {
            let _ = child.kill().await;

            PingOutcome::Error(format!("Cannot read ping output: {error}",))
        }

        Err(_) => {
            let _ = child.kill().await;

            PingOutcome::Error("Ping exceeded the execution deadline.".into())
        }
    }
}

fn parse_output(code: Option<i32>, stdout: &[u8], stderr: &[u8]) -> PingOutcome {
    match code {
        Some(0) => {
            let text = String::from_utf8_lossy(stdout);

            let latency_ms = text
                .lines()
                .find(|line| line.contains("min/avg/max"))
                .and_then(|line| line.split_once('='))
                .and_then(|(_, values)| values.trim().split('/').nth(1))
                .and_then(|value| value.parse::<f64>().ok())
                .filter(|value| value.is_finite() && *value >= 0.0);

            // Missing latency does not turn a reply into packet loss.
            PingOutcome::Reply { latency_ms }
        }

        Some(1) => PingOutcome::NoReply,

        _ => {
            let message: String = String::from_utf8_lossy(stderr)
                .chars()
                .filter(|character| !character.is_control())
                .take(240)
                .collect();

            PingOutcome::Error(if message.is_empty() {
                format!("Ping failed with exit code {code:?}")
            } else {
                message
            })
        }
    }
}
