use std::{fmt, net::IpAddr};

/// A validated IP address or hostname.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target(String);

impl Target {
    pub fn parse(input: &str) -> Result<Self, String> {
        let value = input.trim();

        if let Ok(ip) = value.parse::<IpAddr>() {
            return Ok(Self(ip.to_string()));
        }

        let hostname = value.strip_suffix('.').unwrap_or(value);

        let invalid_numeric_ip = hostname.contains('.')
            && hostname
                .bytes()
                .all(|byte| byte.is_ascii_digit() || byte == b'.');

        let valid = !hostname.is_empty()
            && hostname.len() <= 253
            && !invalid_numeric_ip
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
            return Err("Enter an IPv4/IPv6 address or ASCII hostname, \
                 without a URL or port."
                .into());
        }

        // Preserve the root dot of absolute DNS names.
        Ok(Self(value.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
