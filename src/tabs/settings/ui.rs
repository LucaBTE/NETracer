use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::theme::{self, ThemeId};

use super::SettingsTab;

pub(super) fn render(frame: &mut Frame, area: Rect, tab: &mut SettingsTab) {
    let show_intro = area.height >= 12;
    let show_preview = area.height >= 16;
    let areas = Layout::vertical([
        Constraint::Length(if show_intro { 3 } else { 0 }),
        Constraint::Min(4),
        Constraint::Length(if show_preview { 5 } else { 0 }),
    ])
    .split(area);
    let (intro, table, preview) = (areas[0], areas[1], areas[2]);
    let palette = theme::current();

    if show_intro {
        frame.render_widget(
            Paragraph::new(
                "Choose a visual profile. Changes are previewed immediately and kept for this session.",
            )
            .style(Style::default().fg(palette.text).bg(palette.panel))
            .block(panel("[ APPEARANCE // THEME SELECTOR ]")),
            intro,
        );
    }

    let compact = area.width < 60;
    let rows = ThemeId::ALL.iter().map(|candidate| {
        let marker = if *candidate == theme::active() {
            "● ACTIVE"
        } else {
            ""
        };
        let name = Cell::from(candidate.name()).style(
            Style::default()
                .fg(palette.text)
                .add_modifier(Modifier::BOLD),
        );
        let state = Cell::from(marker).style(Style::default().fg(palette.green));
        if compact {
            Row::new(vec![name, state])
        } else {
            Row::new(vec![
                name,
                Cell::from(candidate.description()).style(Style::default().fg(palette.muted)),
                state,
            ])
        }
        .style(Style::default().bg(palette.panel))
    });
    let widths = if compact {
        vec![Constraint::Fill(1), Constraint::Length(10)]
    } else {
        vec![
            Constraint::Length(18),
            Constraint::Fill(1),
            Constraint::Length(10),
        ]
    };
    let headers = if compact {
        vec![" THEME", "STATE"]
    } else {
        vec![" THEME", "DESCRIPTION", "STATE"]
    };
    let widget = Table::new(rows, widths)
        .header(
            Row::new(headers).style(
                Style::default()
                    .fg(palette.void)
                    .bg(palette.text)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .row_highlight_style(
            Style::default()
                .fg(palette.text)
                .bg(palette.grid)
                .add_modifier(Modifier::BOLD),
        )
        .block(panel("[ INSTALLED THEMES ]"));
    tab.table_area = table;
    frame.render_stateful_widget(widget, table, &mut tab.table_state);

    if show_preview {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("██ ", Style::default().fg(palette.cyan)),
                    Span::styled("DATA  ", Style::default().fg(palette.text)),
                    Span::styled("██ ", Style::default().fg(palette.orange)),
                    Span::styled("ACTION  ", Style::default().fg(palette.text)),
                    Span::styled("██ ", Style::default().fg(palette.green)),
                    Span::styled("ONLINE  ", Style::default().fg(palette.text)),
                    Span::styled("██ ", Style::default().fg(palette.red)),
                    Span::styled("ERROR", Style::default().fg(palette.text)),
                ]),
                Line::from(Span::styled(
                    format!("Current profile: {}", theme::active().name()),
                    Style::default().fg(palette.muted),
                )),
            ])
            .style(Style::default().bg(palette.panel_active))
            .block(panel("[ LIVE COLOR TEST ]")),
            preview,
        );
    }
}

fn panel(title: &'static str) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::current().grid))
}
