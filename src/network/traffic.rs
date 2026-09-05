use std::io;

use netdev::get_interfaces;

#[derive(Clone, Copy, Debug)]
pub struct Counters {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct InterfaceSnapshot {
    pub name: String,
    pub is_default: bool,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub gateway: Option<String>,
    pub counters: Option<Counters>,
}

/// Collect interface metadata and cumulative OS traffic counters off the UI thread.
pub async fn read_interfaces() -> io::Result<Vec<InterfaceSnapshot>> {
    tokio::task::spawn_blocking(|| {
        let mut interfaces = get_interfaces()
            .into_iter()
            .map(|interface| InterfaceSnapshot {
                name: interface.name,
                is_default: interface.default,
                ipv4: interface
                    .ipv4
                    .iter()
                    .map(|network| network.addr().to_string())
                    .collect(),
                ipv6: interface
                    .ipv6
                    .iter()
                    .map(|network| network.addr().to_string())
                    .collect(),
                gateway: interface
                    .gateway
                    .as_ref()
                    .and_then(|gateway| gateway.ipv4.first())
                    .map(ToString::to_string),
                counters: interface.stats.map(|stats| Counters {
                    rx_bytes: stats.rx_bytes,
                    tx_bytes: stats.tx_bytes,
                }),
            })
            .collect::<Vec<_>>();

        interfaces.sort_by(|left, right| {
            right
                .is_default
                .cmp(&left.is_default)
                .then_with(|| left.name.cmp(&right.name))
        });

        interfaces
    })
    .await
    .map_err(|error| io::Error::other(format!("Interface collector failed: {error}")))
}
