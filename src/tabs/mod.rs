pub mod overview;
pub mod ping;

use crate::component::Component;

/// The only registration point for built-in tabs and source-level extensions.
pub fn builtins() -> Vec<Box<dyn Component>> {
    vec![
        Box::new(overview::OverviewTab::new()),
        Box::new(ping::PingTab::new()),
    ]
}
