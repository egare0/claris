# Claris

A clean, minimal, and fluent logging framework for Rust. Zero-dependency by default.
Built on top of the standard 'log' facade.

## Features

- **Zero Dependency:** Only relies on the official 'log' crate. No bloat.
- **Fluent Builder API:** Easy to configure with method chaining.
- **Module-Level Filtering:** Set different log levels for specific modules or crates.
- **Beautiful Output:** Formatted with ANSI escape sequences for clarity, keeping output perfectly aligned.

## Installation

Add this to your 'Cargo.toml':

```toml
[dependencies]
log = "0.4"
claris = "0.1.0"
```

## Usage

Claris uses a builder pattern to configure the logger before initializing it globally.

```rust
use claris::Claris;
use log::LevelFilter;

fn main() {
    Claris::new()
        .with_level(LevelFilter::Info)
        // Mute noisy crates, only allow their errors
        .with_module_level("wgpu", LevelFilter::Error)
        // Allow deep debugging for a specific internal module
        .with_module_level("my_module::core", LevelFilter::Trace)
        .init()
        .unwrap();
    
    log::info!("Claris initialized successfully.");
    log::warn!("This is a warning.");
    log::error!("Critical failure!");
    
    // This will only print if you configured the specific module or global level
    log::debug!("Debugging internal states.");
}
```

## License

Dual-licensed under either of:
- MIT License ('LICENSE-MIT')
- Apache License, Version 2.0 ('LICENSE-APACHE')