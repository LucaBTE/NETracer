use ratatui::style::{Color, Modifier, Style};

// Muted, print-like colors inspired by 1970s control panels. Keeping every
// channel away from full intensity prevents the interface from looking neon.
pub const VOID: Color = Color::Rgb(24, 23, 22);
pub const PANEL: Color = Color::Rgb(31, 30, 28);
pub const PANEL_ACTIVE: Color = Color::Rgb(41, 39, 36);
pub const GRID: Color = Color::Rgb(88, 88, 80);
pub const MUTED: Color = Color::Rgb(137, 136, 123);
pub const TEXT: Color = Color::Rgb(207, 201, 180);
pub const CYAN: Color = Color::Rgb(105, 157, 158);
pub const ORANGE: Color = Color::Rgb(190, 160, 105);
pub const RED: Color = Color::Rgb(180, 103, 89);
pub const GREEN: Color = Color::Rgb(126, 154, 133);

pub fn base() -> Style {
    Style::default().fg(TEXT).bg(VOID)
}

pub fn label() -> Style {
    Style::default().fg(CYAN).add_modifier(Modifier::BOLD)
}
