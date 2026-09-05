mod ui;

use std::time::Instant;

use ratatui::{Frame, layout::Rect};

use crate::{components::Component, network::interfaces::NetworkInfo};

pub struct OverviewTab {
    network: NetworkInfo,
    started_at: Instant,
}

impl OverviewTab {
    pub fn new() -> Self {
        Self {
            network: NetworkInfo::discover(),
            started_at: Instant::now(),
        }
    }
}

impl Default for OverviewTab {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for OverviewTab {
    fn id(&self) -> &'static str {
        "overview"
    }

    fn title(&self) -> &'static str {
        "Overview"
    }

    fn help(&self) -> &'static str {
        "Q/ESC: quit"
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        ui::render(frame, area, &self.network, self.started_at.elapsed());
    }
}
