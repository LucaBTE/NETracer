use std::time::{Duration, Instant};

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};

use crate::theme;

use super::model::{Rates, TrafficModel, TrafficSeries};

pub(super) fn render(
    frame: &mut Frame,
    area: Rect,
    hostname: &str,
    uptime: Duration,
    traffic: &TrafficModel,
    list_area: &mut Rect,
    list_offset: &mut usize,
) {
    if area.height < 10 || area.width < 42 {
        render_tiny(frame, area, hostname, traffic, list_area, list_offset);
        return;
    }

    let sidebar_width = if area.width < 60 { 16 } else { 21 };
    let [interfaces, dashboard] =
        Layout::horizontal([Constraint::Length(sidebar_width), Constraint::Min(26)]).areas(area);
    let information_height = if area.height < 16 { 4 } else { 6 };
    let [information, graphs, statistics] = Layout::vertical([
        Constraint::Length(information_height),
        Constraint::Min(3),
        Constraint::Length(5),
    ])
    .areas(dashboard);
    let graph_areas = if graphs.height < 9 {
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(graphs)
    } else {
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(graphs)
    };
    let (rx_graph, tx_graph) = (graph_areas[0], graph_areas[1]);

    render_interfaces(frame, interfaces, traffic, list_offset);
    *list_area = interfaces;
    render_information(frame, information, hostname, uptime, traffic);

    let series = traffic.selected_series();
    render_chart(frame, rx_graph, series, true);
    render_chart(frame, tx_graph, series, false);
    render_statistics(frame, statistics, series);
}

fn render_tiny(
    frame: &mut Frame,
    area: Rect,
    hostname: &str,
    traffic: &TrafficModel,
    list_area: &mut Rect,
    list_offset: &mut usize,
) {
    let list_width = (area.width / 2).clamp(12, 18);
    let [interfaces, summary] =
        Layout::horizontal([Constraint::Length(list_width), Constraint::Min(12)]).areas(area);
    render_interfaces(frame, interfaces, traffic, list_offset);
    *list_area = interfaces;

    let palette = theme::current();
    let series = traffic.selected_series();
    let rx = series
        .current
        .map(|rates| format_rate(rates.rx_bytes_per_second))
        .unwrap_or_else(|| "N/A".into());
    let tx = series
        .current
        .map(|rates| format_rate(rates.tx_bytes_per_second))
        .unwrap_or_else(|| "N/A".into());
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                traffic.selected_name().to_owned(),
                theme::label(),
            )),
            Line::from(format!("RX  {rx}")),
            Line::from(format!("TX  {tx}")),
            Line::from(Span::styled(
                hostname.to_owned(),
                Style::default().fg(palette.muted),
            )),
        ])
        .style(Style::default().fg(palette.text).bg(palette.panel))
        .block(panel("[ LIVE TRAFFIC ]")),
        summary,
    );
}

fn render_interfaces(frame: &mut Frame, area: Rect, traffic: &TrafficModel, offset: &mut usize) {
    let palette = theme::current();
    let visible = usize::from(area.height.saturating_sub(2)).max(1);
    let count = traffic.choices().count();
    *offset = traffic
        .selected_index()
        .saturating_sub(visible - 1)
        .min(count.saturating_sub(visible));

    let rows = traffic
        .choices()
        .enumerate()
        .skip(*offset)
        .take(visible)
        .map(|(index, name)| {
            let selected = index == traffic.selected_index();
            let available = index == 0 || traffic.interface(name).is_some();
            let marker = if selected {
                "▶"
            } else if available {
                "●"
            } else {
                "○"
            };
            let style = if selected {
                Style::default()
                    .fg(palette.text)
                    .bg(palette.grid)
                    .add_modifier(Modifier::BOLD)
            } else if available {
                Style::default().fg(palette.text)
            } else {
                Style::default().fg(palette.muted)
            };
            Line::from(format!(" {marker} {name}")).style(style)
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        Paragraph::new(rows)
            .style(Style::default().bg(palette.panel))
            .block(panel("[ TRAFFIC SOURCES ]")),
        area,
    );
}

fn render_information(
    frame: &mut Frame,
    area: Rect,
    hostname: &str,
    uptime: Duration,
    traffic: &TrafficModel,
) {
    let palette = theme::current();
    let interface = traffic.selected_interface();
    let configured = interface.is_some_and(|item| !item.ipv4.is_empty() || !item.ipv6.is_empty());
    let (state, state_color) = if traffic.is_all() {
        ("AGGREGATE", palette.cyan)
    } else if interface.is_none() {
        ("UNAVAILABLE", palette.red)
    } else if configured {
        ("CONFIGURED", palette.green)
    } else {
        ("NO ADDRESS", palette.orange)
    };

    let address = interface
        .and_then(|item| item.ipv4.first().or_else(|| item.ipv6.first()))
        .map(String::as_str)
        .unwrap_or(if traffic.is_all() {
            "All interfaces"
        } else {
            "Not available"
        });

    let gateway = interface
        .and_then(|item| item.gateway.as_deref())
        .unwrap_or(if traffic.is_all() {
            "Multiple / none"
        } else {
            "Not available"
        });

    let mut lines = vec![
        Line::from(vec![
            field("SOURCE", traffic.selected_name(), palette.cyan),
            field("STATE", state, state_color),
        ]),
        Line::from(vec![
            field("HOST", hostname, palette.text),
            field("UPTIME", &format_duration(uptime), palette.muted),
        ]),
        Line::from(vec![
            field("ADDR", address, palette.text),
            field("GW", gateway, palette.text),
        ]),
        Line::from(vec![
            Span::styled(" STATUS ", Style::default().fg(palette.muted)),
            Span::styled(
                traffic.selected_series().status.as_str(),
                Style::default().fg(palette.muted),
            ),
        ]),
    ];
    lines.truncate(usize::from(area.height.saturating_sub(2)));
    frame.render_widget(
        Paragraph::new(lines)
            .style(Style::default().bg(palette.panel))
            .block(panel("[ SOURCE INFORMATION ]")),
        area,
    );
}

fn render_chart(frame: &mut Frame, area: Rect, series: &TrafficSeries, receive: bool) {
    let palette = theme::current();
    let maximum = series
        .history()
        .iter()
        .filter_map(|point| point.rates)
        .map(|rates| rate_value(rates, receive))
        .fold(0.0_f64, f64::max);
    let (divisor, unit) = rate_scale(maximum);
    let upper = (maximum / divisor).max(1.0) * 1.1;
    let segments = graph_segments(series, receive, divisor);
    let color = if receive {
        palette.cyan
    } else {
        palette.orange
    };
    let datasets = segments
        .iter()
        .map(|segment| {
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(color))
                .data(segment)
        })
        .collect::<Vec<_>>();

    frame.render_widget(
        Chart::new(datasets)
            .block(panel(if receive {
                "[ RX // LAST 120 SECONDS ]"
            } else {
                "[ TX // LAST 120 SECONDS ]"
            }))
            .x_axis(
                Axis::default()
                    .bounds([0.0, 120.0])
                    .style(Style::default().fg(palette.grid))
                    .labels(["-120s", "-60s", "now"]),
            )
            .y_axis(
                Axis::default()
                    .title(unit)
                    .bounds([0.0, upper])
                    .style(Style::default().fg(palette.muted))
                    .labels(["0".to_owned(), format_value(upper)]),
            ),
        area,
    );
}

fn render_statistics(frame: &mut Frame, area: Rect, series: &TrafficSeries) {
    let [rx, tx] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    frame.render_widget(
        metric_line(
            series.current.map(|rates| rates.rx_bytes_per_second),
            series.total_rx,
            series.peak_rx,
            theme::current().cyan,
        )
        .block(panel("[ RX RECEIVED ]")),
        rx,
    );
    frame.render_widget(
        metric_line(
            series.current.map(|rates| rates.tx_bytes_per_second),
            series.total_tx,
            series.peak_tx,
            theme::current().orange,
        )
        .block(panel("[ TX TRANSMITTED ]")),
        tx,
    );
}

fn metric_line(current: Option<f64>, total: u64, peak: f64, color: Color) -> Paragraph<'static> {
    let palette = theme::current();
    let current = current.map(format_rate).unwrap_or_else(|| "N/A".into());

    Paragraph::new(vec![
        Line::from(Span::styled(
            format!(" NOW   {current}"),
            Style::default().fg(color).bold(),
        )),
        Line::from(Span::styled(
            format!(" TOTAL {}", format_bytes(total)),
            Style::default().fg(palette.text),
        )),
        Line::from(Span::styled(
            format!(" PEAK  {}", format_rate(peak)),
            Style::default().fg(palette.text),
        )),
    ])
}

fn graph_segments(series: &TrafficSeries, receive: bool, divisor: f64) -> Vec<Vec<(f64, f64)>> {
    let now = Instant::now();
    let mut segments = Vec::new();
    let mut current = Vec::new();
    for point in series.history() {
        if let Some(rates) = point.rates {
            let age = now.saturating_duration_since(point.at).as_secs_f64();
            if age <= 120.0 {
                current.push((120.0 - age, rate_value(rates, receive) / divisor));
            }
        } else if !current.is_empty() {
            segments.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

fn rate_value(rates: Rates, receive: bool) -> f64 {
    if receive {
        rates.rx_bytes_per_second
    } else {
        rates.tx_bytes_per_second
    }
}

fn rate_scale(value: f64) -> (f64, &'static str) {
    if value >= 1024.0 * 1024.0 {
        (1024.0 * 1024.0, "MiB/s")
    } else {
        (1024.0, "KiB/s")
    }
}

fn format_rate(value: f64) -> String {
    let (divisor, unit) = rate_scale(value);
    format!("{:.1} {unit}", value / divisor)
}

fn format_bytes(bytes: u64) -> String {
    const MIB: f64 = 1024.0 * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    if bytes as f64 >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB)
    } else {
        format!("{:.2} MiB", bytes as f64 / MIB)
    }
}

fn format_value(value: f64) -> String {
    if value >= 10.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
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

fn field(label: &'static str, value: &str, color: Color) -> Span<'static> {
    Span::styled(format!(" {label} {value}  "), Style::default().fg(color))
}

fn panel(title: &'static str) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::current().grid))
        .style(Style::default().bg(theme::current().panel))
}
