use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    //divides into three main areas

    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let uptime = app.uptime().as_secs();

    let interface = app.network.interface.as_deref().unwrap_or("N/A");
    let ipv4 = app.network.ipv4.as_deref().unwrap_or("N/A");
    let gateway = app.network.gateway.as_deref().unwrap_or("N/A");

    let link_status = if app.network.has_link() {
        "ACTIVE"
    } else {
        "UNAVAILABLE"
    };

    let header = Paragraph::new("NETracer // NETWORK DIAGNOSIS")
        .alignment(Alignment::Center)
        .block(Block::new().borders(Borders::ALL));

    let ping_status = if app.ping.success {
        "ONLINE"
    } else {
        "OFFLINE"
    };

    let latency = app
        .ping
        .latency_ms
        .map(|value| format!("{value:.1} ms"))
        .unwrap_or_else(|| "N/A".to_string());

    let body = Paragraph::new(format!(
        "\
    \nSYSTEM READY

    HOSTNAME     {}
    INTERFACE    {}
    IPv4         {}
    GATEWAY      {}
    LINK         {}

    TARGET       1.1.1.1
    STATUS       {}
    LATENCY      {}

    UPTIME       {}s",
        app.network.hostname, interface, ipv4, gateway, link_status, ping_status, latency, uptime,
    ))
    .alignment(Alignment::Center)
    .block(Block::new().title(" NODE STATUS ").borders(Borders::ALL));

    let footer = Paragraph::new("[Q] Quit   [ESC] Exit")
        .alignment(Alignment::Center)
        .block(Block::new().borders(Borders::ALL));

    frame.render_widget(header, header_area);
    frame.render_widget(body, body_area);
    frame.render_widget(footer, footer_area);
}
