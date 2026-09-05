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
const ITALIC_LOGO: [&str; 3] = [
    r"  _______                 ",
    r" /| )(_   /  \_ _ _ _ _ ",
    r"/ |/ /__ (  / (/( (-/   ",
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

    if frame.area().width < 30 || frame.area().height < 10 {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled("[ DISPLAY LINK FAILURE ]", theme::label())),
                Line::from(Span::styled(
                    "Resize the terminal  //  Ctrl+C to quit",
                    Style::default().fg(theme::current().muted),
                )),
            ])
            .alignment(Alignment::Center),
            frame.area(),
        );
        return Vec::new();
    }

    let masthead_height = if frame.area().height >= 27 && frame.area().width >= 50 {
        5
    } else if frame.area().height >= 20 && frame.area().width >= 38 {
        4
    } else {
        1
    };
    let footer_height = if frame.area().height >= 18 { 3 } else { 1 };
    let areas = Layout::vertical([
        Constraint::Length(masthead_height),
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(footer_height),
    ])
    .split(frame.area());
    let (masthead, tabs, body, footer) = (areas[0], areas[1], areas[2], areas[3]);

    let logo_rows: &[&str] = if masthead_height == 5 {
        &LOGO
    } else if masthead_height == 4 {
        &ITALIC_LOGO
    } else {
        &["N E T R A C E R  //  NETWORK DIAGNOSTICS"]
    };
    let logo = logo_rows
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

    let compact_width = frame.area().width < 55;
    let tab_constraints = components.iter().map(|component| {
        if component.id() == "settings" {
            Constraint::Length(if compact_width { 10 } else { 18 })
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
        let title = if compact_width {
            match component.id() {
                "overview" => "OVR",
                "settings" => "SET",
                _ => "PING",
            }
        } else {
            component.title()
        };
        frame.render_widget(
            Paragraph::new(format!(" {prefix} {} {} ", index + 1, title.to_uppercase()))
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
    let footer_widget = Paragraph::new(footer_text);
    if footer_height == 3 {
        frame.render_widget(
            footer_widget.block(
                Block::default()
                    .title("[ COMMAND LINE ]")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme::current().grid)),
            ),
            footer,
        );
    } else {
        frame.render_widget(footer_widget, footer);
    }

    tab_areas
}
