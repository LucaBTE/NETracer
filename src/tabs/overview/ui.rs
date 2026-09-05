use std::time::Duration;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::{network::interfaces::NetworkInfo, theme};

pub(super) fn render(frame: &mut Frame, area: Rect, network: &NetworkInfo, uptime: Duration) {
    let [summary, content] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).areas(area);
    let [identity, connection] =
        Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)]).areas(content);

    let (status, status_color) = if network.has_link() {
        ("ONLINE", theme::GREEN)
    } else {
        ("UNAVAILABLE", theme::RED)
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " NETWORK STATUS  ",
                Style::default().fg(theme::VOID).bg(status_color).bold(),
            ),
            Span::styled(
                format!("  {status}"),
                Style::default().fg(status_color).bold(),
            ),
            Span::styled("    SESSION  ", theme::label()),
            Span::styled(format_duration(uptime), Style::default().fg(theme::TEXT)),
        ]))
        .style(Style::default().bg(theme::PANEL))
        .block(panel("[ SYSTEM STATUS ]")),
        summary,
    );

    frame.render_widget(
        Paragraph::new(vec![
            field("Hostname", &network.hostname),
            Line::default(),
            Line::from(Span::styled(
                "LOCAL NODE // STARTUP SNAPSHOT",
                Style::default().fg(theme::MUTED),
            )),
        ])
        .style(Style::default().bg(theme::PANEL))
        .block(panel("[ NODE IDENTITY ]").padding(Padding::new(1, 1, 1, 0))),
        identity,
    );

    frame.render_widget(
        Paragraph::new(vec![
            field(
                "Interface",
                network.interface.as_deref().unwrap_or("Not detected"),
            ),
            field(
                "IPv4 address",
                network.ipv4.as_deref().unwrap_or("Not assigned"),
            ),
            field(
                "Gateway",
                network.gateway.as_deref().unwrap_or("Not detected"),
            ),
            Line::default(),
            Line::from(vec![
                Span::styled("● ", Style::default().fg(status_color)),
                Span::styled(
                    if network.has_link() {
                        "Interface configured"
                    } else {
                        "No configured network interface"
                    },
                    Style::default().fg(theme::TEXT),
                ),
            ]),
            Line::from(Span::styled(
                "  HOST ─── INTERFACE ─── GATEWAY ─── WAN",
                Style::default().fg(theme::MUTED),
            )),
        ])
        .style(Style::default().bg(theme::PANEL))
        .block(panel("[ NETWORK LINK ]").padding(Padding::new(1, 1, 1, 0))),
        connection,
    );
}

fn panel(title: &'static str) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::GRID))
}

fn field(label: &'static str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<14}"), Style::default().fg(theme::MUTED)),
        Span::styled(value.to_owned(), Style::default().fg(theme::CYAN).bold()),
    ])
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        (seconds / 60) % 60,
        seconds % 60
    )
}
