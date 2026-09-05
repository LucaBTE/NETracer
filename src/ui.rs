use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::component::Component;

pub(crate) fn render(frame: &mut Frame, components: &mut [Box<dyn Component>], active: usize) -> Vec<Rect> {
    for component in components.iter_mut() {
        component.reset_layout();
    }
    if frame.area().width < 65 || frame.area().height < 22 {
        frame.render_widget(
            Paragraph::new("Resize to at least 65 columns x 22 rows. Ctrl+C: Quit"),
            frame.area(),
        );
        return Vec::new();
    }
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3), Constraint::Min(1), Constraint::Length(4),
    ]).areas(frame.area());

    let block = Block::default().title(" NETracer // NETWORK DIAGNOSIS ").borders(Borders::ALL);
    let inner = block.inner(header);
    frame.render_widget(block, header);

    let constraints = vec![Constraint::Fill(1); components.len()];
    let tab_areas = Layout::horizontal(constraints).split(inner).to_vec();
    for (index, component) in components.iter().enumerate() {
        let style = if index == active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        frame.render_widget(
            Paragraph::new(format!("[{}] {}", index + 1, component.title())).style(style),
            tab_areas[index],
        );
    }

    components[active].render(frame, body);
    let help = format!("{}\nTab/Shift+Tab: Switch tab   Ctrl+C: Quit", components[active].help());
    frame.render_widget(
        Paragraph::new(help).wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL)),
        footer,
    );
    tab_areas
}
