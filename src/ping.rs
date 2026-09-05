use std::{
    net::IpAddr,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub type PingResult = Result<Option<f64>, String>;

pub fn validate_target(input: &str) -> Result<String, String> {
    let target = input.trim();

    if target.parse::<IpAddr>().is_ok() {
        return Ok(target.to_string());
    }

    let hostname = target.trim_end_matches('.');

    let valid = !hostname.is_empty()
        && hostname.len() <= 253
        && hostname.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        });

    if !valid {
        return Err("Enter an IP address or hostname, without a URL or port.".into());
    }

    Ok(hostname.to_ascii_lowercase())
}

pub fn ping(target: &str) -> PingResult {
    // These arguments target Linux iputils ping.
    let mut child = Command::new("ping")
        .env("LC_ALL", "C")
        .args(["-n", "-c", "1", "-W", "1", "-w", "2", target])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Cannot start ping: {error}"))?;

    let started = Instant::now();

    // Bound the entire operation, including hostname resolution.
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() < Duration::from_secs(5) => {
                thread::sleep(Duration::from_millis(25));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("Ping exceeded the 5-second execution limit.".into());
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("Cannot monitor ping: {error}"));
            }
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("Cannot read ping output: {error}"))?;

    match output.status.code() {
        Some(0) => {
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Read the average from the min/avg/max/mdev summary.
            let latency = stdout
                .lines()
                .find(|line| line.contains("min/avg/max"))
                .and_then(|line| line.split_once('='))
                .and_then(|(_, values)| values.trim().split('/').nth(1))
                .and_then(|value| value.parse::<f64>().ok())
                .filter(|value| value.is_finite() && *value >= 0.0);

            latency
                .map(Some)
                .ok_or_else(|| "Reply received, but latency could not be parsed.".into())
        }
        Some(1) => Ok(None),
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr);

            let message: String = stderr
                .chars()
                .filter(|character| !character.is_control())
                .take(240)
                .collect();

            Err(if message.is_empty() {
                format!("Ping failed: {}", output.status)
            } else {
                message
            })
        }
    }
}
