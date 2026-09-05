use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, Wrap},
};

use crate::{network::ping::PingOutcome, theme};

use super::{
    PingTab,
    model::{PingEntry, ProbeState},
};

pub(super) fn render(frame: &mut Frame, area: Rect, tab: &mut PingTab) {
    let show_details = area.height >= 14;
    let details_height = if show_details { 6 } else { 0 };
    let areas = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(details_height),
        Constraint::Length(1),
    ])
    .split(area);
    let (input, table, details, message) = (areas[0], areas[1], areas[2], areas[3]);

    tab.input_area = input;
    tab.table_area = table;

    render_input(frame, input, tab);
    render_table(frame, table, tab);

    let text = tab
        .table_state
        .selected()
        .and_then(|index| tab.entries.get(index))
        .map(details_text)
        .unwrap_or_else(|| "No saved targets. Press / to enter an address or hostname.".into());

    if show_details {
        frame.render_widget(
            Paragraph::new(text)
                .style(
                    Style::default()
                        .fg(theme::current().text)
                        .bg(theme::current().panel_active),
                )
                .wrap(Wrap { trim: false })
                .block(
                    Block::default()
                        .title("[ TARGET TELEMETRY ]")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme::current().grid)),
                ),
            details,
        );
    }

    frame.render_widget(
        Paragraph::new(tab.message.as_str())
            .style(
                Style::default()
                    .fg(theme::current().orange)
                    .bg(theme::current().void)
                    .bold(),
            )
            .wrap(Wrap { trim: false }),
        message,
    );
}

fn render_input(frame: &mut Frame, area: Rect, tab: &PingTab) {
    let width = usize::from(area.width.saturating_sub(2));

    let scroll = tab.input.visual_scroll(width.saturating_sub(1));

    let placeholder = tab.input.value().is_empty() && !tab.editing;

    let text = if placeholder {
        "Press / or click here to enter an IP address or hostname"
    } else {
        tab.input.value()
    };

    let border_color = if tab.editing {
        theme::current().cyan
    } else {
        theme::current().muted
    };

    frame.render_widget(
        Paragraph::new(text)
            .style(
                Style::default()
                    .fg(if tab.editing {
                        theme::current().cyan
                    } else {
                        theme::current().orange
                    })
                    .bg(theme::current().panel_active),
            )
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .title(if tab.editing {
                        "[ TARGET INPUT // ENTER TO PING ]"
                    } else {
                        "[ TARGET INPUT ]"
                    })
                    .borders(Borders::ALL)
                    .border_type(if tab.editing {
                        BorderType::Thick
                    } else {
                        BorderType::Plain
                    })
                    .border_style(Style::default().fg(border_color)),
            ),
        area,
    );

    if tab.editing && width > 0 {
        let x = tab
            .input
            .visual_cursor()
            .saturating_sub(scroll)
            .min(width - 1) as u16;

        frame.set_cursor_position((area.x + 1 + x, area.y + 1));
    }
}

fn render_table(frame: &mut Frame, area: Rect, tab: &mut PingTab) {
    let compact = area.width < 50;
    let medium = area.width < 70;
    let rows = tab
        .entries
        .iter()
        .map(|entry| {
            let latency = match &entry.state {
                ProbeState::Finished(PingOutcome::Reply { latency_ms }) => *latency_ms,

                _ => None,
            };

            let last_run = entry
                .last_run
                .map(|time| format!("{}s ago", time.elapsed().as_secs(),))
                .unwrap_or_else(|| "-".into());

            let mut cells = vec![
                Cell::from(format!("  {}", entry.target))
                    .style(Style::default().fg(theme::current().text)),
                Cell::from(status(&entry.state)).style(status_style(&entry.state)),
            ];
            if !compact {
                cells.push(
                    Cell::from(milliseconds(latency))
                        .style(Style::default().fg(theme::current().cyan)),
                );
            }
            if !medium {
                cells.push(Cell::from(last_run).style(Style::default().fg(theme::current().muted)));
                cells.push(
                    Cell::from("PING").style(
                        Style::default()
                            .fg(theme::current().orange)
                            .add_modifier(Modifier::BOLD),
                    ),
                );
            }
            Row::new(cells).style(Style::default().bg(theme::current().panel))
        })
        .collect::<Vec<_>>();

    let (headers, widths) = if compact {
        (
            vec!["  TARGET", "STATUS"],
            vec![Constraint::Fill(1), Constraint::Length(15)],
        )
    } else if medium {
        (
            vec!["  TARGET", "STATUS", "LATENCY"],
            vec![
                Constraint::Fill(1),
                Constraint::Length(15),
                Constraint::Length(12),
            ],
        )
    } else {
        (
            vec!["  TARGET", "STATUS", "LATENCY", "LAST RUN", "ACTION"],
            vec![
                Constraint::Fill(1),
                Constraint::Length(15),
                Constraint::Length(12),
                Constraint::Length(11),
                Constraint::Length(8),
            ],
        )
    };
    let table = Table::new(rows, widths)
        .header(
            Row::new(headers).style(
                Style::default()
                    .fg(theme::current().void)
                    .bg(theme::current().text)
                    .bold(),
            ),
        )
        .row_highlight_style(
            Style::default()
                .fg(theme::current().text)
                .bg(theme::current().grid)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .title("[ RECENT TARGETS // PROBE QUEUE ]")
                .title_bottom("[ ↑/↓ SELECT ][ ENTER PING ][ DEL REMOVE ]")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::current().grid)),
        )
        .style(Style::default().bg(theme::current().panel));

    frame.render_stateful_widget(table, area, &mut tab.table_state);
}

fn status(state: &ProbeState) -> &'static str {
    match state {
        ProbeState::Ready => "Ready",

        ProbeState::Running => "Pinging…",

        ProbeState::Finished(PingOutcome::Reply { .. }) => "Reply received",

        ProbeState::Finished(PingOutcome::NoReply) => "No reply",

        ProbeState::Finished(PingOutcome::Error(_)) => "Error",
    }
}

fn status_style(state: &ProbeState) -> Style {
    let color = match state {
        ProbeState::Ready => theme::current().muted,
        ProbeState::Running => theme::current().orange,
        ProbeState::Finished(PingOutcome::Reply { .. }) => theme::current().green,
        ProbeState::Finished(PingOutcome::NoReply | PingOutcome::Error(_)) => theme::current().red,
    };
    Style::default()
        .fg(color)
        .bg(theme::current().panel)
        .add_modifier(Modifier::BOLD)
}

fn milliseconds(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.2} ms"))
        .unwrap_or_else(|| "-".into())
}

fn details_text(entry: &PingEntry) -> String {
    let stats = &entry.statistics;

    let loss = stats
        .loss()
        .map(|value| format!("{value:.1}%"))
        .unwrap_or_else(|| "-".into());

    let detail = match &entry.state {
        ProbeState::Finished(PingOutcome::Error(error)) => error.as_str(),

        ProbeState::Finished(PingOutcome::Reply { latency_ms: None }) => {
            "Reply received; latency unavailable."
        }

        other => status(other),
    };

    format!(
        "Target     {}\n\
         Probes     {} completed  •  {} replies  •  {} loss\n\
         Latency    {} min  /  {} avg  /  {} max\n\
         Status     {}",
        entry.target,
        stats.completed,
        stats.received,
        loss,
        milliseconds(stats.min),
        milliseconds(stats.average()),
        milliseconds(stats.max),
        detail,
    )
}
