use std::io::{self, IsTerminal, Write};
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError, Log};
use crate::color::Color;

mod color;

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
    pub fn new() -> Self {
        Self {
            level: LevelFilter::Info,
            modules: Vec::new(),
            colors_enabled: io::stdout().is_terminal(),
        }
    }

    #[must_use]
    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    #[must_use]
    pub fn with_module_level(mut self, target: &str, level: LevelFilter) -> Self {
        self.modules.push((target.to_string(), level));
        self
    }

    #[must_use]
    pub fn with_colors(mut self, enabled: bool) -> Self {
        self.colors_enabled = enabled;
        self
    }

    pub fn init(self) -> Result<(), SetLoggerError> {
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

struct ClarisLogger {
    level: LevelFilter,
    modules: Vec<(String, LevelFilter)>,
    colors_enabled: bool,
}

impl Log for ClarisLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let target = metadata.target();

        let mut specific_level = None;
        let mut max_len = 0;

        for (module_name, module_level) in &self.modules {
            if module_matches(target, module_name) && module_name.len() > max_len {
                specific_level = Some(*module_level);
                max_len = module_name.len();
            }
        }

        if let Some(level) = specific_level {
            metadata.level() <= level
        } else {
            metadata.level() <= self.level
        }
    }

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

    fn flush(&self) {}
}

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
}