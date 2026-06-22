//! Claris is a minimal, fluent logging implementation built on top of the
//! [`log`](https://docs.rs/log) facade. No async runtime, no config files,
//! no macros beyond what `log` already gives you — just a builder you chain
//! together and call `.init()` on.
//!
//! ```
//! use claris::Claris;
//! use log::LevelFilter;
//!
//! Claris::new()
//!     .with_level(LevelFilter::Info)
//!     .with_module_level("noisy_dep", LevelFilter::Error)
//!     .init()
//!     .unwrap();
//!
//! log::info!("ready");
//! ```
//!
//! Colors are on by default when stdout is a terminal and off otherwise, so
//! piping output to a file won't fill it with ANSI escape codes. Override
//! that with [`Claris::with_colors`] if you need to.

use std::io::{self, IsTerminal, Write};
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError, Log};
use crate::color::Color;

mod color;

/// Builder for configuring and installing the global logger.
///
/// `Claris` itself does nothing until [`init`](Claris::init) is called —
/// it's just a bag of settings that gets handed off to the actual
/// [`Log`] implementation, `ClarisLogger`, at that point.
#[derive(Debug)]
pub struct Claris {
    level: LevelFilter,
    modules: Vec<(String, LevelFilter)>,
    colors_enabled: bool,
}

impl Default for Claris {
    fn default() -> Self {
        Self::new()
    }
}

impl Claris {
    /// Starts a new builder with `Info` as the global level and no
    /// per-module overrides.
    ///
    /// Colors default to on when stdout looks like a real terminal and off
    /// otherwise.
    pub fn new() -> Self {
        Self {
            level: LevelFilter::Info,
            modules: Vec::new(),
            colors_enabled: io::stdout().is_terminal(),
        }
    }

    /// Sets the fallback level for any target that doesn't have a more
    /// specific override from [`with_module_level`](Self::with_module_level).
    #[must_use]
    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    /// Overrides the level for `target` and everything nested under it.
    ///
    /// If you register overlapping overrides (e.g. both `"app"` and `"app::db"`),
    /// the most specific one wins for any given target.
    #[must_use]
    pub fn with_module_level(mut self, target: &str, level: LevelFilter) -> Self {
        self.modules.push((target.to_string(), level));
        self
    }

    /// Forces ANSI colors on or off, overriding the terminal-detection
    /// default from [`new`](Self::new).
    #[must_use]
    pub fn with_colors(mut self, enabled: bool) -> Self {
        self.colors_enabled = enabled;
        self
    }

    /// Builds the logger and installs it as the global `log` backend.
    ///
    /// This can only happen once per process — `log` itself enforces that,
    /// which is what the `Result` here is about.
    ///
    /// # Errors
    /// Returns [`SetLoggerError`] if a logger (Claris or otherwise) has
    /// already been installed.
    pub fn init(self) -> Result<(), SetLoggerError> {
        // log's max_level filter is global and applies before our own
        // per-module filtering even runs, so it has to be at least as
        // permissive as the loosest level anyone asked for, or module
        // overrides that are more verbose than the global level would get
        // silently swallowed upstream.
        let mut max_filter = self.level;
        for (_, module_level) in &self.modules {
            if *module_level > max_filter {
                max_filter = *module_level;
            }
        }

        let logger = ClarisLogger {
            level: self.level,
            modules: self.modules,
            colors_enabled: self.colors_enabled,
        };

        log::set_max_level(max_filter);
        log::set_boxed_logger(Box::new(logger))
    }
}

/// The actual [`Log`] implementation that gets boxed and registered with
/// the `log` facade once [`Claris::init`] runs.
///
/// This is intentionally not public — `Claris` is the only supported way
/// to build one.
struct ClarisLogger {
    level: LevelFilter,
    modules: Vec<(String, LevelFilter)>,
    colors_enabled: bool,
}

impl Log for ClarisLogger {
    /// Decides whether a record should be printed, given the global level
    /// and any per-module overrides.
    ///
    /// We walk every registered module and keep the longest one whose name
    /// matches the record's target, since that's the most specific rule
    /// the caller wrote.
    fn enabled(&self, metadata: &Metadata) -> bool {
        let target = metadata.target();
        let level = self.modules.iter()
            .filter(|(name, _)| module_matches(target, name))
            .max_by_key(|(name, _)| name.len())
            .map_or(self.level, |(_, level)| *level);
        metadata.level() <= level
    }

    /// Writes one log line to stdout, colored or plain depending on
    /// `colors_enabled`.
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let stdout = io::stdout();
        let mut out = stdout.lock();

        let result = if self.colors_enabled {
            let color = match record.level() {
                Level::Error => Color::Red,
                Level::Warn => Color::Yellow,
                Level::Info => Color::Green,
                Level::Debug => Color::Cyan,
                Level::Trace => Color::Magenta
            };

            writeln!(out, "{gray}[{c}{level:<5}{reset}{gray} {target}]{reset} {msg}",
                     gray = Color::Gray.as_str(),
                c = color.as_str(),
                     level = record.level().as_str(),
                     reset = Color::Reset.as_str(),
                     target = record.target(),
                     msg = record.args())
        } else {
            writeln!(out, "[{:<5} {}] {}", record.level().as_str(), record.target(), record.args())
        };

        if let Err(e) = result {
            eprintln!("claris: failed to write log line: {e}");
        }
    }

    /// No-op — every call to `log()` already writes straight to stdout
    /// without internal buffering, so there's nothing to flush.
    fn flush(&self) {}
}

/// True if `target` *is* `module_name`, or is nested under it
/// (`module_name::anything`).
fn module_matches(target: &str, module_name: &str) -> bool {
    target
        .strip_prefix(module_name)
        .map_or(false, |rest| rest.is_empty() || rest.starts_with("::"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn logger(level: LevelFilter, modules: &[(&str, LevelFilter)]) -> ClarisLogger {
        ClarisLogger {
            level,
            modules: modules.iter().map(|(n, l)| (n.to_string(), *l)).collect(),
            colors_enabled: false,
        }
    }

    fn meta(target: &str, level: Level) -> Metadata<'_> {
        Metadata::builder().target(target).level(level).build()
    }

    #[test]
    fn falls_back_to_global_level_without_module_override() {
        let logger = logger(LevelFilter::Warn, &[]);
        assert!(logger.enabled(&meta("anything", Level::Warn)));
        assert!(!logger.enabled(&meta("anything", Level::Info)));
    }

    #[test]
    fn exact_module_match_overrides_global_level() {
        let logger = logger(LevelFilter::Error, &[("wgpu", LevelFilter::Trace)]);
        assert!(logger.enabled(&meta("wgpu", Level::Trace)));
    }

    #[test]
    fn nested_module_match_overrides_global_level() {
        let logger = logger(LevelFilter::Error, &[("wgpu", LevelFilter::Trace)]);
        assert!(logger.enabled(&meta("wgpu::core::device", Level::Trace)));
    }

    #[test]
    fn sibling_crate_with_same_prefix_is_not_matched() {
        let logger = logger(LevelFilter::Error, &[("wgpu", LevelFilter::Trace)]);
        assert!(!logger.enabled(&meta("wgpu_hal", Level::Trace)));
        assert!(!logger.enabled(&meta("wgpu_hal::instance", Level::Trace)));
    }

    #[test]
    fn longest_matching_module_wins() {
        let logger = logger(
            LevelFilter::Error,
            &[("test_crate", LevelFilter::Warn), ("test_crate::deep", LevelFilter::Trace)],
        );
        assert!(logger.enabled(&meta("test_crate::deep::fn", Level::Trace)));
        assert!(!logger.enabled(&meta("test_crate::other", Level::Trace)));
    }

    #[test]
    fn duplicate_module_registration_last_one_wins() {
        let logger = logger(
            LevelFilter::Off,
            &[("wgpu", LevelFilter::Error), ("wgpu", LevelFilter::Trace)],
        );
        assert!(logger.enabled(&meta("wgpu", Level::Trace)));
    }
}