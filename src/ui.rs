use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, Tabs, Wrap},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    app.input_area = Rect::default();
    app.table_area = Rect::default();

    if frame.area().width < 65 || frame.area().height < 22 {
        frame.render_widget(
            Paragraph::new("Resize the terminal to at least 65 columns x 22 rows."),
            frame.area(),
        );
        return;
    }

    let [tabs_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let tabs = Tabs::new(vec!["[1] Overview", "[2] Ping"])
        .select(app.tab)
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .title(" NETracer // NETWORK DIAGNOSIS ")
                .borders(Borders::ALL),
        );

    frame.render_widget(tabs, tabs_area);

    if app.tab == 0 {
        render_overview(frame, app, body_area);
    } else {
        render_ping(frame, app, body_area);
    }

    let help = if app.editing {
        "Enter: Ping   Esc: Cancel input   Ctrl+C: Quit"
    } else if app.tab == 1 {
        "/: New target   Up/Down: Select   Enter/Click: Ping\nDel: Remove   Tab: Switch tab   Q: Quit"
    } else {
        "Tab: Switch tab   2: Ping   Q: Quit"
    };

    frame.render_widget(
        Paragraph::new(help).block(Block::default().borders(Borders::ALL)),
        footer_area,
    );
}

fn render_overview(frame: &mut Frame, app: &App, area: Rect) {
    let interface = app.network.interface.as_deref().unwrap_or("N/A");
    let ipv4 = app.network.ipv4.as_deref().unwrap_or("N/A");
    let gateway = app.network.gateway.as_deref().unwrap_or("N/A");

    let link = if app.network.has_link() {
        "ACTIVE"
    } else {
        "UNAVAILABLE"
    };

    let text = format!(
        "HOSTNAME     {}\n\
         INTERFACE    {}\n\
         IPv4         {}\n\
         GATEWAY      {}\n\
         LINK         {}\n\n\
         UPTIME       {}s",
        app.network.hostname,
        interface,
        ipv4,
        gateway,
        link,
        app.uptime().as_secs(),
    );

    frame.render_widget(
        Paragraph::new(text).block(Block::default().title(" Overview ").borders(Borders::ALL)),
        area,
    );
}

fn render_ping(frame: &mut Frame, app: &mut App, area: Rect) {
    let [input_area, table_area, details_area, message_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(4),
        Constraint::Length(7),
        Constraint::Length(2),
    ])
    .areas(area);

    app.input_area = input_area;
    app.table_area = table_area;

    let input_style = if app.editing {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let input_text = if app.input.is_empty() && !app.editing {
        "Press / or click here to enter an IP address or hostname".to_string()
    } else {
        app.input.clone()
    };

    let inner_width = usize::from(input_area.width.saturating_sub(2));
    let scroll = if app.editing {
        app.input
            .len()
            .saturating_sub(inner_width.saturating_sub(1))
    } else {
        0
    };

    frame.render_widget(
        Paragraph::new(input_text).scroll((0, scroll as u16)).block(
            Block::default()
                .title(" IP address or hostname ")
                .borders(Borders::ALL)
                .border_style(input_style),
        ),
        input_area,
    );

    if app.editing {
        let cursor = app.input.len().saturating_sub(scroll) as u16;
        frame.set_cursor_position((input_area.x + 1 + cursor, input_area.y + 1));
    }

    let rows: Vec<Row<'static>> = app
        .entries
        .iter()
        .map(|entry| {
            let latency = entry
                .latency
                .map(|value| format!("{value:.2} ms"))
                .unwrap_or_else(|| "-".into());

            let last_run = entry
                .last_run
                .map(|time| format!("{}s ago", time.elapsed().as_secs()))
                .unwrap_or_else(|| "-".into());

            let status = if entry.status.starts_with("Error:") {
                "Error".to_string()
            } else {
                entry.status.clone()
            };

            Row::new(vec![
                entry.target.clone(),
                status,
                latency,
                last_run,
                "[Ping]".into(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Fill(1),
            Constraint::Length(15),
            Constraint::Length(12),
            Constraint::Length(11),
            Constraint::Length(6),
        ],
    )
    .header(
        Row::new(vec!["Target", "Status", "Latency", "Last run", "Action"])
            .style(Style::default().fg(Color::Cyan)),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .block(
        Block::default()
            .title(" Recent targets ")
            .borders(Borders::ALL),
    );

    frame.render_stateful_widget(table, table_area, &mut app.table_state);

    let details = app
        .table_state
        .selected()
        .and_then(|index| app.entries.get(index))
        .map(|entry| entry.statistics())
        .unwrap_or_else(|| {
            "No recent targets.\nPress / to enter a destination and run your first ping.".into()
        });

    frame.render_widget(
        Paragraph::new(details).wrap(Wrap { trim: false }).block(
            Block::default()
                .title(" Selected target // session statistics ")
                .borders(Borders::ALL),
        ),
        details_area,
    );

    frame.render_widget(
        Paragraph::new(app.message.as_str())
            .style(Style::default().fg(Color::Yellow))
            .wrap(Wrap { trim: false }),
        message_area,
    );
}
