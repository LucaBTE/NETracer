use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{components::Component, theme};

const LOGO: [&str; 5] = [
    r" _   _ _____ _____                         ",
    r"| \ | | ____|_   _| __ __ _  ___ ___ _ __ ",
    r"|  \| |  _|   | || '__/ _` |/ __/ _ \ '__|",
    r"| |\  | |___  | || | | (_| | (_|  __/ |   ",
    r"|_| \_|_____| |_||_|  \__,_|\___\___|_|   ",
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
                    Style::default().fg(theme::MUTED),
                )),
            ])
            .alignment(Alignment::Center),
            frame.area(),
        );
        return Vec::new();
    }

    let [masthead, tabs, body, footer] = Layout::vertical([
        Constraint::Length(6),
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
                    .fg(theme::TEXT)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(logo)
            .alignment(Alignment::Center)
            .style(Style::default().bg(theme::VOID)),
        masthead,
    );

    let tab_block = Block::default()
        .title(Span::styled(
            "[ NETWORK ENDPOINT TRACER // CONTROL DECK ]",
            Style::default().fg(theme::MUTED),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::GRID))
        .style(Style::default().bg(theme::PANEL));
    let tab_inner = tab_block.inner(tabs);
    frame.render_widget(tab_block, tabs);

    let tab_areas = Layout::horizontal(vec![Constraint::Fill(1); components.len()])
        .split(tab_inner)
        .to_vec();
    for (index, component) in components.iter().enumerate() {
        let (prefix, style) = if index == active {
            (
                ">",
                Style::default()
                    .fg(theme::VOID)
                    .bg(theme::CYAN)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (" ", Style::default().fg(theme::TEXT).bg(theme::PANEL))
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
                .fg(theme::VOID)
                .bg(theme::GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", theme::base()),
        Span::styled(components[active].help(), Style::default().fg(theme::TEXT)),
        Span::styled("  │  TAB: module ", Style::default().fg(theme::MUTED)),
    ]);
    frame.render_widget(
        Paragraph::new(footer_text).block(
            Block::default()
                .title("[ COMMAND LINE ]")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::GRID)),
        ),
        footer,
    );

    tab_areas
}
