# mcman

A Ratatui TUI console for Minecraft servers.

## Features

- Launches `server.jar` (or any jar via first argument) automatically
- Coloured log output (warnings = yellow, errors = red, join/leave = green)
- Tab-completion for all vanilla commands + sub-commands
- Command history (↑/↓ arrows)
- Auto-scroll with manual override (mousewheel or PgUp/PgDn)
- Auto-scroll re-engages when you scroll back to the bottom
- Cursor movement in the input bar (←/→, Home, End)

## Build

Requires Rust + Cargo (https://rustup.rs) and Java in your PATH.

```bash
cargo install mcman
-OR-
cargo install --path <path-to-project>
```

## Usage

```bash
# Uses server.jar in current directory
mcman

# Specify a different jar
mcman /path/to/server.jar
```

## Keybindings

| Key           | Action                     |
| ------------- | -------------------------- |
| Enter         | Send command               |
| Tab           | Cycle tab completions      |
| ↑ / ↓         | Command history            |
| PgUp / PgDn   | Scroll logs (10 lines)     |
| Mouse wheel   | Scroll logs (3 lines)      |
| ← / →         | Move cursor in input       |
| Home / End    | Jump to start/end of input |
| Backspace/Del | Delete character           |
| Ctrl+C        | Send `stop` then quit      |

## Metacommands

| Command | Action                  |
| ------- | ----------------------- |
| :clear  | Clear the log buffer.   |
| :exit   | Exit after sending stop |
| :quit   | Aliased to `:exit`      |

## Notes

- Commands can be typed with or without a leading `/`.
- Autocomplete for most common commands.
- The status indicator (● RUNNING / ○ STOPPED) reflects the server process state.
- The scroll indicator shows current position when not auto-scrolling.
