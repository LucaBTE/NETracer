pub mod overview;
pub mod ping;
pub mod settings;

use crate::components::Component;

pub fn builtins() -> Vec<Box<dyn Component>> {
    vec![
        Box::new(overview::OverviewTab::new()),
        Box::new(ping::PingTab::new()),
        Box::new(settings::SettingsTab::new()),
    ]
}
