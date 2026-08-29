use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(frame: &mut Frame) {
    //divides into three main areas

    let [header_area, body_area, footer_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let header = Paragraph::new("NETracer // NETWORK DIAGNOSIS")
        .alignment(Alignment::Center)
        .block(Block::new().borders(Borders::ALL));

    let body =
        Paragraph::new("\nSYSTEM READY\n\nNetwork diagnostics module awaiting initialization.")
            .alignment(Alignment::Center)
            .block(Block::new().title(" NODE STATUS ").borders(Borders::ALL));

    let footer = Paragraph::new("[Q] Quit   [ESC] Exit")
        .alignment(Alignment::Center)
        .block(Block::new().borders(Borders::ALL));

    frame.render_widget(header, header_area);
    frame.render_widget(body, body_area);
    frame.render_widget(footer, footer_area);
}
