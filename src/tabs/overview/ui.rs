use std::time::Duration;
use ratatui::{Frame, layout::Rect, widgets::{Block, Borders, Paragraph}};
use crate::network::interfaces::NetworkInfo;

pub(super) fn render(frame: &mut Frame, area: Rect, network: &NetworkInfo, uptime: Duration) {
    let text = format!(
        "HOSTNAME     {}\nINTERFACE    {}\nIPv4         {}\nGATEWAY      {}\nNETWORK      {}\n\nUPTIME       {}s",
        network.hostname,
        network.interface.as_deref().unwrap_or("N/A"),
        network.ipv4.as_deref().unwrap_or("N/A"),
        network.gateway.as_deref().unwrap_or("N/A"),
        if network.has_link() { "CONFIGURED" } else { "UNAVAILABLE" },
        uptime.as_secs(),
    );
    frame.render_widget(
        Paragraph::new(text).block(Block::default().title(" Overview // startup snapshot ").borders(Borders::ALL)),
        area,
    );
}
