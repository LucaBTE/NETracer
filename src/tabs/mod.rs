pub mod overview;
pub mod ping;

use crate::components::Component;

pub fn builtins() -> Vec<Box<dyn Component>> {
    vec![
        Box::new(overview::OverviewTab::new()),
        Box::new(ping::PingTab::new()),
    ]
}
