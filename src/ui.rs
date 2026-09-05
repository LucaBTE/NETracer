use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{components::Component, theme};

const LOGO: [&str; 4] = [
    "   _  ____________                    ",
    "  / |/ / __/_  __/______ ________ ____",
    " /    / _/  / / / __/ _ `/ __/ -_) __/",
    "/_/|_/___/ /_/ /_/  \\_,_/\\__/\\__/_/   ",
];

pub(crate) fn render(
    frame: &mut Frame,
    components: &mut [Box<dyn Component>],
    active: usize,
) -> Vec<Rect> {
    frame.render_widget(Block::default().style(theme::base()), frame.area());
    for component in components.iter_mut() {
        component.reset_layout();
    }

    if frame.area().width < 65 || frame.area().height < 26 {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled("[ DISPLAY LINK FAILURE ]", theme::label())),
                Line::from(Span::styled(
                    "MINIMUM GRID: 65 × 26  //  CTRL+C TO ABORT",
                    Style::default().fg(theme::current().muted),
                )),
            ])
            .alignment(Alignment::Center),
            frame.area(),
        );
        return Vec::new();
    }

    let [masthead, tabs, body, footer] = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(3),
        Constraint::Min(14),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let logo = LOGO
        .iter()
        .map(|row| {
            Line::from(Span::styled(
                *row,
                Style::default()
                    .fg(theme::current().text)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(logo)
            .alignment(Alignment::Center)
            .style(Style::default().bg(theme::current().void)),
        masthead,
    );

    let tab_block = Block::default()
        .title(Span::styled(
            "[ NETWORK ENDPOINT TRACER // CONTROL DECK ]",
            Style::default().fg(theme::current().muted),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::current().grid))
        .style(Style::default().bg(theme::current().panel));
    let tab_inner = tab_block.inner(tabs);
    frame.render_widget(tab_block, tabs);

    let tab_constraints = components.iter().map(|component| {
        if component.id() == "settings" {
            Constraint::Length(18)
        } else {
            Constraint::Fill(1)
        }
    });
    let tab_areas = Layout::horizontal(tab_constraints)
        .split(tab_inner)
        .to_vec();
    for (index, component) in components.iter().enumerate() {
        let (prefix, style) = if index == active {
            (
                ">",
                Style::default()
                    .fg(theme::current().void)
                    .bg(theme::current().cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (
                " ",
                Style::default()
                    .fg(theme::current().text)
                    .bg(theme::current().panel),
            )
        };
        frame.render_widget(
            Paragraph::new(format!(
                " {prefix} F{}  {} ",
                index + 1,
                component.title().to_uppercase()
            ))
            .style(style)
            .alignment(Alignment::Center),
            tab_areas[index],
        );
    }

    components[active].render(frame, body);

    let footer_text = Line::from(vec![
        Span::styled(
            " READY ",
            Style::default()
                .fg(theme::current().void)
                .bg(theme::current().green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", theme::base()),
        Span::styled(
            components[active].help(),
            Style::default().fg(theme::current().text),
        ),
        Span::styled(
            "  │  TAB: module ",
            Style::default().fg(theme::current().muted),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(footer_text).block(
            Block::default()
                .title("[ COMMAND LINE ]")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::current().grid)),
        ),
        footer,
    );

    tab_areas
}
