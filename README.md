# claris

A minimal, fluent logging implementation built on top of the
[`log`](https://docs.rs/log) facade. No async runtime, no config files,
no macros beyond what `log` already gives you.

The only dependency is the `log` facade itself, which your application
almost certainly already has.

## Installation
```toml
[dependencies]
log = "0.4"
claris = "0.1.1"
```

## Usage
```rust
use claris::Claris;
use log::LevelFilter;

fn main() {
    Claris::new()
        .with_level(LevelFilter::Info)
        .with_module_level("noisy_dep", LevelFilter::Error)
        .init()
        .unwrap();

    log::info!("ready");
}
```

## Features

**Fluent builder.** `Claris::new()` returns a builder — chain `with_*`
methods, call `.init()` once.

**Per-module level overrides.** `with_module_level("wgpu", LevelFilter::Error)`
silences a noisy dependency without touching your own logs. Matching happens
on `::` boundaries, so `"wgpu"` affects `wgpu` and `wgpu::core` but not
`wgpu_hal` — they share a prefix, not a namespace. If multiple overrides
match the same target, the most specific one wins.

**Automatic color detection.** Colors are on when stdout is an interactive
terminal and off otherwise, so redirecting output to a file or a CI log
won't fill it with ANSI escape codes. Override this with `.with_colors(bool)`
if you need to.

**Zero-allocation hot path.** No heap allocations per log call — level
strings and ANSI color codes are `&'static str` and written directly into
the output, so logging doesn't put pressure on the allocator.

## Log Format
```bash
[INFO  my_crate::module] message here
```

Colors are applied to the level and the surrounding brackets when stdout
is a terminal.

## License

Licensed under either of [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE) at your option.