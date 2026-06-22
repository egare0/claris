//! ANSI color codes used to paint log levels when writing to a terminal.
//!
//! Kept deliberately tiny and private — this isn't meant to be a general
//! purpose terminal-color crate, just enough to make `claris`'s own output
//! readable.

/// The handful of ANSI colors `claris` actually uses for log levels, plus
/// `Reset` to turn them back off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Used for `Error` level lines.
    Red,
    /// Used for `Warn` level lines.
    Yellow,
    /// Used for `Info` level lines.
    Green,
    /// Used for `Debug` level lines.
    Cyan,
    /// Used for `Trace` level lines.
    Magenta,
    /// Used for the brackets and target name around the level, so they
    /// read as "metadata" rather than the message itself.
    Gray,
    /// Turns any preceding color back off. Every colored segment needs to
    /// be followed by this, or the color bleeds into whatever the
    /// terminal prints next.
    Reset
}

impl Color {
    /// The raw ANSI escape sequence for this color.
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