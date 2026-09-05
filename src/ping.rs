use std::process::Command;

pub struct PingResult {
    pub latency_ms: Option<f64>, //float 64 bit. If present: value. If not present: None
    pub success: bool,
}

//get string (housename or id) and return a struct with latency time and success status.
pub fn ping(target: &str) -> PingResult {
    //execute ping command and caputure its output and the exit status.
    let output = Command::new("ping")
        .args(["-c", "1", "-W", "1", target])
        .output();

    //if the ping command doesn't runs, return a failed struct with no latency and false as success status
    let Ok(output) = output else {
        return PingResult {
            latency_ms: None,
            success: false,
        };
    };

    //if the ping runs but the host doesn't exist or doesnt respond, return a failed struct with no latency and false as success status
    if !output.status.success() {
        return PingResult {
            latency_ms: None,
            success: false,
        };
    }

    //get output bites and converts them in a string. from_utf8_lossy is used to avoid error while reading bad bites
    let stdout = String::from_utf8_lossy(&output.stdout);

    //get stdout string, look for 'time=' and extract the value after it in f64, if not: None
    let latency_ms = stdout
        .split("time=")
        .nth(1)
        .and_then(|part| part.split_whitespace().next())
        .and_then(|value| value.parse::<f64>().ok());

    //assign the values
    PingResult {
        latency_ms,
        success: true,
    }
}
