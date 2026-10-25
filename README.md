# taqvim

A Persian calendar daemon with systray icon support.

## Features

- **Systray icon**: Displays the current Persian day number as 5x7 bitmap font
- **Real-time updates**: Icon updates when the day changes
- **Quit functionality**: Quit button in systray menu cleanly terminates the daemon
- **Keyboard shortcuts**:
  - `Ctrl+T` or `t` key: Go to today
  - `Ctrl+Left`/`Ctrl+Right`: Navigate days
  - `Ctrl+Q` or `q`: Quit the application

## Building

```bash
cargo build --release
```

## Running

```bash
# Run in daemon mode (with systray)
./target/debug/taqvim --daemon

# Run in terminal mode
./target/debug/taqvim
```

## Files Edited

- `src/tray.rs` - Added 5x7 bitmap font rendering for Persian day numbers, changed channel type to `u32`
- `src/main.rs` - Updated to send `u32` day numbers through the tray channel instead of tooltip strings
- `Cargo.toml` - Bumped version to 0.1.5

## How It Works

1. The daemon runs in the background, updating the systray icon every 2 seconds
2. The tray icon displays the current Persian day number using a custom 5x7 bitmap font
3. When the day changes, `tray.set_icon()` is called with the new bitmap
4. The Quit menu item sends a menu event that sets an atomic quit flag
5. The daemon loop checks this flag, removes the PID file, and exits cleanly
