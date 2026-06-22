// Demonstrates the basics: a global level, a per-module override, and
// how colors behave. Run with `cargo run --example basic`.
use claris::Claris;

fn main() {
    Claris::new()
        .with_level(log::LevelFilter::Info)
        // Ignore the global level entirely for anything under
        // test_crate, show everything down to trace.
        .with_module_level("test_crate", log::LevelFilter::Trace)
        .init()
        .unwrap();

    log::info!("Starting basic example");
    log::warn!("This is a warning");
    log::error!("This is an error");
    log::debug!("This is a debug but you can't see it");
    log::trace!("This is a trace but you can't see it");

    // These are logged under the test_crate target, so the
    // module-level override kicks in and overrides the global
    // Info level.
    log::debug!(target: "test_crate", "This is a debug in test_crate, so you can see it");
    log::trace!(target: "test_crate", "This is a trace in test_crate, so you can see it");
}