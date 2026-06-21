use claris::Claris;

fn main() {
    Claris::new()
        .with_level(log::LevelFilter::Info)
        .with_module_level("test_crate", log::LevelFilter::Trace)
        .init()
        .unwrap();

    log::info!("Starting basic example");
    log::warn!("This is a warning");
    log::error!("This is an error");
    log::debug!("This is a debug but you can't see it");
    log::trace!("This is a trace but you can't see it");

    log::debug!(target: "test_crate", "This is a debug in test_crate, so you can see it");
    log::trace!(target: "test_crate", "This is a trace in test_crate, so you can see it");
}