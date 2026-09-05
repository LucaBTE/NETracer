use std::sync::atomic::{AtomicUsize, Ordering};

use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeId {
    DustTerminal,
    AmberArchive,
    Vampire,
    Unicorn,
    IceStation,
    DosClassic,
}

impl ThemeId {
    pub const ALL: [Self; 6] = [
        Self::DustTerminal,
        Self::AmberArchive,
        Self::Vampire,
        Self::Unicorn,
        Self::IceStation,
        Self::DosClassic,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::DustTerminal => "Dust Terminal",
            Self::AmberArchive => "Amber Archive",
            Self::Vampire => "Vampire",
            Self::Unicorn => "Unicorn",
            Self::IceStation => "Ice Station",
            Self::DosClassic => "DOS Classic",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::DustTerminal => "Warm charcoal, aged ivory and muted instruments",
            Self::AmberArchive => "Monochrome amber CRT recovered from storage",
            Self::Vampire => "Black console with restrained blood-red signals",
            Self::Unicorn => "Midnight violet, dusty pink and electric blue",
            Self::IceStation => "Cold steel interface with arctic cyan telemetry",
            Self::DosClassic => "Cobalt blue and grey, straight out of the 1990s",
        }
    }

    const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub void: Color,
    pub panel: Color,
    pub panel_active: Color,
    pub grid: Color,
    pub muted: Color,
    pub text: Color,
    pub cyan: Color,
    pub orange: Color,
    pub red: Color,
    pub green: Color,
}

const PALETTES: [Palette; 6] = [
    Palette {
        void: Color::Rgb(24, 23, 22),
        panel: Color::Rgb(31, 30, 28),
        panel_active: Color::Rgb(41, 39, 36),
        grid: Color::Rgb(88, 88, 80),
        muted: Color::Rgb(137, 136, 123),
        text: Color::Rgb(207, 201, 180),
        cyan: Color::Rgb(105, 157, 158),
        orange: Color::Rgb(190, 160, 105),
        red: Color::Rgb(180, 103, 89),
        green: Color::Rgb(126, 154, 133),
    },
    Palette {
        void: Color::Rgb(24, 17, 9),
        panel: Color::Rgb(34, 24, 12),
        panel_active: Color::Rgb(46, 32, 15),
        grid: Color::Rgb(105, 69, 27),
        muted: Color::Rgb(154, 105, 48),
        text: Color::Rgb(221, 169, 87),
        cyan: Color::Rgb(226, 151, 52),
        orange: Color::Rgb(240, 184, 79),
        red: Color::Rgb(191, 82, 49),
        green: Color::Rgb(192, 167, 79),
    },
    Palette {
        void: Color::Rgb(17, 8, 11),
        panel: Color::Rgb(27, 11, 15),
        panel_active: Color::Rgb(40, 14, 20),
        grid: Color::Rgb(92, 36, 45),
        muted: Color::Rgb(139, 67, 76),
        text: Color::Rgb(211, 180, 177),
        cyan: Color::Rgb(181, 104, 105),
        orange: Color::Rgb(195, 132, 92),
        red: Color::Rgb(218, 52, 70),
        green: Color::Rgb(153, 151, 103),
    },
    Palette {
        void: Color::Rgb(18, 11, 28),
        panel: Color::Rgb(29, 16, 42),
        panel_active: Color::Rgb(43, 21, 57),
        grid: Color::Rgb(82, 55, 104),
        muted: Color::Rgb(137, 101, 151),
        text: Color::Rgb(214, 190, 211),
        cyan: Color::Rgb(80, 166, 181),
        orange: Color::Rgb(210, 145, 173),
        red: Color::Rgb(202, 85, 137),
        green: Color::Rgb(139, 169, 151),
    },
    Palette {
        void: Color::Rgb(13, 18, 21),
        panel: Color::Rgb(19, 27, 31),
        panel_active: Color::Rgb(27, 38, 43),
        grid: Color::Rgb(65, 91, 99),
        muted: Color::Rgb(108, 137, 143),
        text: Color::Rgb(191, 207, 207),
        cyan: Color::Rgb(91, 168, 177),
        orange: Color::Rgb(184, 158, 111),
        red: Color::Rgb(188, 97, 102),
        green: Color::Rgb(113, 161, 145),
    },
    Palette {
        void: Color::Rgb(0, 0, 118),
        panel: Color::Rgb(0, 0, 148),
        panel_active: Color::Rgb(0, 0, 178),
        grid: Color::Rgb(118, 118, 174),
        muted: Color::Rgb(170, 170, 202),
        text: Color::Rgb(218, 218, 218),
        cyan: Color::Rgb(92, 200, 200),
        orange: Color::Rgb(218, 190, 105),
        red: Color::Rgb(225, 113, 113),
        green: Color::Rgb(135, 202, 135),
    },
];

static ACTIVE: AtomicUsize = AtomicUsize::new(0);

pub fn active() -> ThemeId {
    ThemeId::ALL[ACTIVE.load(Ordering::Relaxed).min(ThemeId::ALL.len() - 1)]
}

pub fn select(theme: ThemeId) {
    ACTIVE.store(theme.index(), Ordering::Relaxed);
}

pub fn current() -> Palette {
    PALETTES[active().index()]
}

pub fn base() -> Style {
    let palette = current();
    Style::default().fg(palette.text).bg(palette.void)
}

pub fn label() -> Style {
    Style::default()
        .fg(current().cyan)
        .add_modifier(Modifier::BOLD)
}
