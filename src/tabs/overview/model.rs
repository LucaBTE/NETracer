use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use crate::network::traffic::{Counters, InterfaceSnapshot};

const HISTORY_WINDOW: Duration = Duration::from_secs(120);
const MAX_HISTORY_POINTS: usize = 121;

#[derive(Clone, Copy, Debug)]
pub struct Rates {
    pub rx_bytes_per_second: f64,
    pub tx_bytes_per_second: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct HistoryPoint {
    pub at: Instant,
    pub rates: Option<Rates>,
}

pub struct TrafficSeries {
    baseline: Option<(Instant, Counters)>,
    history: VecDeque<HistoryPoint>,
    pub current: Option<Rates>,
    pub total_rx: u64,
    pub total_tx: u64,
    pub peak_rx: f64,
    pub peak_tx: f64,
    pub status: String,
}

impl TrafficSeries {
    fn new(status: &str) -> Self {
        Self {
            baseline: None,
            history: VecDeque::new(),
            current: None,
            total_rx: 0,
            total_tx: 0,
            peak_rx: 0.0,
            peak_tx: 0.0,
            status: status.into(),
        }
    }

    fn record(&mut self, at: Instant, counters: Option<Counters>) -> Option<(Rates, u64, u64)> {
        let Some(counters) = counters else {
            self.baseline = None;
            self.current = None;
            self.push_history(at, None);
            self.status = "Interface or traffic counters unavailable".into();
            return None;
        };

        let Some((previous_at, previous)) = self.baseline.replace((at, counters)) else {
            self.current = None;
            self.push_history(at, None);
            self.status = "Baseline established; waiting for the next sample".into();
            return None;
        };

        let elapsed = at.saturating_duration_since(previous_at).as_secs_f64();
        if elapsed <= f64::EPSILON
            || counters.rx_bytes < previous.rx_bytes
            || counters.tx_bytes < previous.tx_bytes
        {
            self.current = None;
            self.push_history(at, None);
            self.status = "Counters reset; baseline re-established".into();
            return None;
        }

        let delta_rx = counters.rx_bytes - previous.rx_bytes;
        let delta_tx = counters.tx_bytes - previous.tx_bytes;
        let rates = Rates {
            rx_bytes_per_second: delta_rx as f64 / elapsed,
            tx_bytes_per_second: delta_tx as f64 / elapsed,
        };
        self.total_rx = self.total_rx.saturating_add(delta_rx);
        self.total_tx = self.total_tx.saturating_add(delta_tx);
        self.peak_rx = self.peak_rx.max(rates.rx_bytes_per_second);
        self.peak_tx = self.peak_tx.max(rates.tx_bytes_per_second);
        self.current = Some(rates);
        self.push_history(at, Some(rates));
        self.status = "Traffic monitoring active".into();
        Some((rates, delta_rx, delta_tx))
    }

    fn record_aggregate(&mut self, at: Instant, samples: &[(Rates, u64, u64)], total: usize) {
        if samples.is_empty() {
            self.current = None;
            self.push_history(at, None);
            self.status = "Waiting for interface samples".into();
            return;
        }

        let rates = samples.iter().fold(
            Rates {
                rx_bytes_per_second: 0.0,
                tx_bytes_per_second: 0.0,
            },
            |sum, (rates, _, _)| Rates {
                rx_bytes_per_second: sum.rx_bytes_per_second + rates.rx_bytes_per_second,
                tx_bytes_per_second: sum.tx_bytes_per_second + rates.tx_bytes_per_second,
            },
        );
        self.total_rx = self
            .total_rx
            .saturating_add(samples.iter().map(|(_, rx, _)| rx).sum());
        self.total_tx = self
            .total_tx
            .saturating_add(samples.iter().map(|(_, _, tx)| tx).sum());
        self.peak_rx = self.peak_rx.max(rates.rx_bytes_per_second);
        self.peak_tx = self.peak_tx.max(rates.tx_bytes_per_second);
        self.current = Some(rates);
        self.push_history(at, Some(rates));
        self.status = format!(
            "Monitoring {}/{total} interfaces. VPNs and bridges may count traffic twice",
            samples.len()
        );
    }

    pub fn history(&self) -> &VecDeque<HistoryPoint> {
        &self.history
    }

    fn push_history(&mut self, at: Instant, rates: Option<Rates>) {
        self.history.push_back(HistoryPoint { at, rates });
        while self
            .history
            .front()
            .is_some_and(|point| at.saturating_duration_since(point.at) > HISTORY_WINDOW)
        {
            self.history.pop_front();
        }
        while self.history.len() > MAX_HISTORY_POINTS {
            self.history.pop_front();
        }
    }
}

pub struct TrafficModel {
    selected: usize,
    preferred: Option<String>,
    interface_names: Vec<String>,
    interfaces: Vec<InterfaceSnapshot>,
    series: HashMap<String, TrafficSeries>,
    all: TrafficSeries,
}

impl TrafficModel {
    pub fn new(default_interface: Option<String>) -> Self {
        Self {
            selected: 0,
            preferred: default_interface,
            interface_names: Vec::new(),
            interfaces: Vec::new(),
            series: HashMap::new(),
            all: TrafficSeries::new("Waiting for interface samples"),
        }
    }

    pub fn choices(&self) -> impl Iterator<Item = &str> {
        std::iter::once("All").chain(self.interface_names.iter().map(String::as_str))
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_name(&self) -> &str {
        if self.selected == 0 {
            "All"
        } else {
            self.interface_names
                .get(self.selected - 1)
                .map(String::as_str)
                .unwrap_or("All")
        }
    }

    pub fn is_all(&self) -> bool {
        self.selected == 0
    }

    pub fn selected_interface(&self) -> Option<&InterfaceSnapshot> {
        let name = self.interface_names.get(self.selected.checked_sub(1)?)?;
        self.interfaces
            .iter()
            .find(|interface| &interface.name == name)
    }

    pub fn interface(&self, name: &str) -> Option<&InterfaceSnapshot> {
        self.interfaces
            .iter()
            .find(|interface| interface.name == name)
    }

    pub fn selected_series(&self) -> &TrafficSeries {
        if self.selected == 0 {
            &self.all
        } else {
            self.interface_names
                .get(self.selected - 1)
                .and_then(|name| self.series.get(name))
                .unwrap_or(&self.all)
        }
    }

    pub fn select(&mut self, index: usize) {
        self.selected = index.min(self.interface_names.len());
    }

    pub fn move_selection(&mut self, direction: isize) {
        self.select(
            self.selected
                .saturating_add_signed(direction)
                .min(self.interface_names.len()),
        );
    }

    pub fn record_collection(&mut self, at: Instant, interfaces: Vec<InterfaceSnapshot>) {
        self.interfaces = interfaces;
        for interface in &self.interfaces {
            if !self.interface_names.contains(&interface.name) {
                self.interface_names.push(interface.name.clone());
            }
            self.series
                .entry(interface.name.clone())
                .or_insert_with(|| TrafficSeries::new("Waiting for the first sample"));
        }

        if let Some(preferred) = self.preferred.take()
            && let Some(position) = self
                .interface_names
                .iter()
                .position(|name| name == &preferred)
        {
            self.interface_names.swap(0, position);
        }

        let mut aggregate = Vec::new();
        for name in &self.interface_names {
            let counters = self
                .interfaces
                .iter()
                .find(|interface| &interface.name == name)
                .and_then(|interface| interface.counters);
            if let Some(sample) = self
                .series
                .get_mut(name)
                .expect("series created for every known interface")
                .record(at, counters)
            {
                aggregate.push(sample);
            }
        }
        self.all
            .record_aggregate(at, &aggregate, self.interface_names.len());
    }

    pub fn record_failure(&mut self, at: Instant, message: String) {
        for series in self.series.values_mut() {
            series.record(at, None);
        }
        self.all.current = None;
        self.all.push_history(at, None);
        self.all.status = format!("Data unavailable: {message}");
    }
}
