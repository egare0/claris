pub enum Color {
    Red,
    Yellow,
    Green,
    Cyan,
    Magenta,
    Gray,
    Reset
}

impl Color {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Color::Red => "\x1b[31m",
            Color::Yellow => "\x1b[33m",
            Color::Green => "\x1b[32m",
            Color::Cyan => "\x1b[36m",
            Color::Magenta => "\x1B[95m",
            Color::Gray => "\x1b[90m",
            Color::Reset => "\x1b[0m"
        }
    }
}

pub fn paint(color: Color, text: &str) -> String {
    format!("{}{}{}", color.as_str(), text, Color::Reset.as_str())
}