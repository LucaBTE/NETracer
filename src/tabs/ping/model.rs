use std::time::Instant;

use crate::network::{ping::PingOutcome, target::Target};

#[derive(Debug, Default)]
pub(super) struct Statistics {
    pub completed: u64,
    pub received: u64,
    pub samples: u64,
    pub sum: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl Statistics {
    pub fn record(&mut self, outcome: &PingOutcome) {
        match outcome {
            PingOutcome::Reply { latency_ms } => {
                self.completed += 1;
                self.received += 1;

                if let Some(latency) = latency_ms {
                    self.samples += 1;
                    self.sum += latency;

                    self.min = Some(self.min.map_or(*latency, |value| value.min(*latency)));

                    self.max = Some(self.max.map_or(*latency, |value| value.max(*latency)));
                }
            }

            PingOutcome::NoReply => {
                self.completed += 1;
            }

            PingOutcome::Error(_) => {}
        }
    }

    pub fn average(&self) -> Option<f64> {
        (self.samples > 0).then(|| self.sum / self.samples as f64)
    }

    pub fn loss(&self) -> Option<f64> {
        (self.completed > 0)
            .then(|| 100.0 * (self.completed - self.received) as f64 / self.completed as f64)
    }
}

#[derive(Debug)]
pub(super) enum ProbeState {
    Ready,
    Running,
    Finished(PingOutcome),
}

#[derive(Debug)]
pub(super) struct PingEntry {
    pub target: Target,
    pub state: ProbeState,
    pub statistics: Statistics,
    pub last_run: Option<Instant>,
}

impl PingEntry {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            state: ProbeState::Ready,
            statistics: Statistics::default(),
            last_run: None,
        }
    }

    pub fn start(&mut self) {
        self.state = ProbeState::Running;
        self.last_run = Some(Instant::now());
    }

    pub fn finish(&mut self, outcome: PingOutcome) {
        self.statistics.record(&outcome);
        self.state = ProbeState::Finished(outcome);
    }
}
