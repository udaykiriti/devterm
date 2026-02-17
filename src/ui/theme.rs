use ratatui::style::Color;

// btop-inspired modern color scheme
pub const BG: Color = Color::Rgb(13, 17, 23);
pub const PANEL_BG: Color = Color::Rgb(22, 27, 34);
pub const PANEL_BG_ACTIVE: Color = Color::Rgb(31, 38, 48);
pub const TEXT: Color = Color::Rgb(230, 237, 243);
pub const TEXT_DIM: Color = Color::Rgb(139, 148, 158);
pub const MUTED: Color = Color::Rgb(87, 96, 106);

// Modern accent colors
pub const ACCENT: Color = Color::Rgb(88, 166, 255);
pub const ACCENT_BRIGHT: Color = Color::Rgb(121, 192, 255);
pub const SECONDARY: Color = Color::Rgb(163, 122, 255);
pub const TERTIARY: Color = Color::Rgb(242, 105, 255);

// Status colors with better contrast
pub const GOOD: Color = Color::Rgb(63, 185, 80);
pub const GOOD_BRIGHT: Color = Color::Rgb(86, 217, 105);
pub const WARN: Color = Color::Rgb(255, 184, 0);
pub const WARN_BRIGHT: Color = Color::Rgb(255, 214, 91);
pub const BAD: Color = Color::Rgb(255, 85, 85);
pub const BAD_BRIGHT: Color = Color::Rgb(255, 120, 120);

// UI elements
pub const HIGHLIGHT_BG: Color = Color::Rgb(48, 54, 61);
pub const BORDER: Color = Color::Rgb(48, 54, 61);
pub const BORDER_ACTIVE: Color = Color::Rgb(88, 166, 255);
pub const BORDER_FOCUSED: Color = Color::Rgb(163, 122, 255);

// Special effects
pub const GLOW: Color = Color::Rgb(163, 255, 209);
