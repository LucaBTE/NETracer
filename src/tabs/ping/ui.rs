use ratatui::{
    Frame, layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
};
use crate::network::ping::PingOutcome;
use super::{PingTab, model::{PingEntry, ProbeState}};

pub(super) fn render(frame: &mut Frame, area: Rect, tab: &mut PingTab) {
    let [input, table, details, message] = Layout::vertical([
        Constraint::Length(3), Constraint::Min(3),
        Constraint::Length(7), Constraint::Length(2),
    ]).areas(area);
    tab.input_area = input;
    tab.table_area = table;
    render_input(frame, input, tab);
    render_table(frame, table, tab);

    let text = tab.table_state.selected().and_then(|index| tab.entries.get(index))
        .map(details_text).unwrap_or_else(|| "No recent targets. Press / to enter a destination.".into());
    frame.render_widget(
        Paragraph::new(text).wrap(Wrap { trim: false }).block(
            Block::default().title(" Selected target // session statistics ").borders(Borders::ALL)
        ), details,
    );
    frame.render_widget(
        Paragraph::new(tab.message.as_str()).style(Style::default().fg(Color::Yellow))
            .wrap(Wrap { trim: false }),
        message,
    );
}

fn render_input(frame: &mut Frame, area: Rect, tab: &PingTab) {
    let width = usize::from(area.width.saturating_sub(2));
    let scroll = tab.input.visual_scroll(width.saturating_sub(1));
    let placeholder = tab.input.value().is_empty() && !tab.editing;
    let text = if placeholder { "Press / or click here to enter an IP address or hostname" } else { tab.input.value() };
    frame.render_widget(
        Paragraph::new(text).scroll((0, scroll as u16)).block(
            Block::default().title(" IP address or hostname ").borders(Borders::ALL)
                .border_style(Style::default().fg(if tab.editing { Color::Cyan } else { Color::Reset }))
        ), area,
    );
    if tab.editing && width > 0 {
        let x = tab.input.visual_cursor().saturating_sub(scroll).min(width - 1) as u16;
        frame.set_cursor_position((area.x + 1 + x, area.y + 1));
    }
}

fn render_table(frame: &mut Frame, area: Rect, tab: &mut PingTab) {
    let rows = tab.entries.iter().map(|entry| {
        let latency = match &entry.state {
            ProbeState::Finished(PingOutcome::Reply { latency_ms }) => *latency_ms,
            _ => None,
        };
        Row::new(vec![
            entry.target.to_string(), status(&entry.state).into(), milliseconds(latency),
            entry.last_run.map(|time| format!("{}s ago", time.elapsed().as_secs())).unwrap_or_else(|| "-".into()),
            "[Ping]".into(),
        ])
    }).collect::<Vec<_>>();
    let table = Table::new(rows, [
        Constraint::Fill(1), Constraint::Length(15), Constraint::Length(12),
        Constraint::Length(11), Constraint::Length(6),
    ])
        .header(Row::new(["Target", "Status", "Latency", "Last run", "Action"]).style(Style::default().fg(Color::Cyan)))
        .row_highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .block(Block::default().title(" Recent targets ").borders(Borders::ALL));
    frame.render_stateful_widget(table, area, &mut tab.table_state);
}

fn status(state: &ProbeState) -> &'static str {
    match state {
        ProbeState::Ready => "Ready",
        ProbeState::Running => "Running...",
        ProbeState::Finished(PingOutcome::Reply { .. }) => "Reply received",
        ProbeState::Finished(PingOutcome::NoReply) => "No reply",
        ProbeState::Finished(PingOutcome::Error(_)) => "Error",
    }
}

fn milliseconds(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.2} ms")).unwrap_or_else(|| "-".into())
}

fn details_text(entry: &PingEntry) -> String {
    let stats = &entry.statistics;
    let loss = stats.loss().map(|value| format!("{value:.1}%")).unwrap_or_else(|| "-".into());
    let detail = match &entry.state {
        ProbeState::Finished(PingOutcome::Error(error)) => error.as_str(),
        ProbeState::Finished(PingOutcome::Reply { latency_ms: None }) => "Reply received; latency unavailable.",
        other => status(other),
    };
    format!(
        "{}\nCompleted probes: {}   Replies: {}   Loss: {}\nMin / Avg / Max: {} / {} / {}\n{}",
        entry.target, stats.completed, stats.received, loss,
        milliseconds(stats.min), milliseconds(stats.average()), milliseconds(stats.max), detail,
    )
}
