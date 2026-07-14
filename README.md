# Steamy

Steamy is a small macOS menu-bar app that keeps the display and the system awake.
One click starts or stops macOS's built-in `caffeinate` process.

## Features

- Prevents display sleep with `caffeinate -d`.
- Prevents idle system sleep with `caffeinate -i`.
- Automatically releases both assertions when Steamy exits.
- Provides a right-click menu for a clean exit.
- Uses macOS template icons that adapt to light and dark mode.
- Waits for events instead of continuously polling.

## Requirements

- macOS
- Rust and Cargo

No additional runtime tools are required. `/usr/bin/caffeinate` is included with
macOS.

## Run locally

```shell
cargo run
```

Steamy appears in the macOS menu bar without a Dock icon. Left-click the cup to
toggle keep-awake mode. Right-click it and select **Quit Steamy** to exit.

## Development checks

```shell
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

## Release build

```shell
cargo build --release
```

`cargo build --release` creates an optimized executable, not a signed and
notarized `.app` bundle. Bundling, code signing, notarization, and universal
Apple Silicon/Intel builds are separate distribution steps.

## How it works

Steamy uses a Tao event loop with `ControlFlow::Wait`, so it sleeps until macOS
delivers an event. The `App` struct owns the tray icon, both visual states, and
the optional `caffeinate` child process.

When enabled, Steamy runs:

```text
/usr/bin/caffeinate -d -i -w <steamy-pid>
```

The `-w` option ties `caffeinate` to Steamy's process ID. If Steamy exits,
including through `Ctrl+C`, `caffeinate` exits and releases its assertions.

The PNG source files are embedded into the executable at compile time. They are
scaled to a Retina-sized 36×36 RGBA image during startup and are not required as
separate runtime files. This one-time resize reduced the measured idle resident
memory from 80,864 KiB to 54,704 KiB on the development machine.
