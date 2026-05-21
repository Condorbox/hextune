# `hextune`

Lightweight terminal music player (CLI + interactive TUI).

`hextune` scans a folder on disk, builds a local library (stored as JSON in your OS config
directory), and lets you play music either via simple terminal playback or a full-screen
browser UI.

## Features

- Play a single audio file: `hextune play <FILE>`
- Local library scanning: set a root folder (`path`) and scan it (`refresh`)
- Library playback: `playlist` (simple terminal UI with playback controls)
- Full-screen interactive browser: `browse` (TUI)
- Fuzzy search across **title**, **artist**, and **album**
- Sorting by **title**, **artist**, **album**, or **duration**
- Shuffle + repeat modes + volume + previous-track restart threshold, persisted between runs
- Supported extensions: **mp3**, **flac**, **wav**, **ogg**

## Install

### Pre-built binaries (recommended)

**Linux & macOS:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Condorbox/hextune/releases/download/v1.0.0/hextune-installer.sh | sh
```

**Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/Condorbox/hextune/releases/download/v1.0.0/hextune-installer.ps1 | iex"
```

Or download the binary directly from the [latest release](https://github.com/Condorbox/hextune/releases/latest) for your platform:
- `hextune-x86_64-unknown-linux-gnu.tar.gz` — Linux x86_64
- `hextune-aarch64-unknown-linux-gnu.tar.gz` — Linux ARM64
- `hextune-x86_64-apple-darwin.tar.gz` — macOS Intel
- `hextune-aarch64-apple-darwin.tar.gz` — macOS Apple Silicon
- `hextune-x86_64-pc-windows-msvc.zip` — Windows x86_64

### From crates.io

```bash
cargo install hextune
```

### From source

```bash
# Build a release binary
cargo build --release
./target/release/hextune --help
```

Or install locally from the repo:

```bash
cargo install --path .
hextune --help
```

## Quick start

1) Point `hextune` at your music folder:

```bash
hextune path /path/to/your/music
```

2) Scan it to build/update the library:

```bash
hextune refresh
```

3) Start playback:

```bash
# Full-screen interactive browser (recommended)
hextune browse

# Or: simple “play through the library” mode
hextune playlist
```

## Commands

`hextune --help` shows the full help text. These are the available subcommands:

- `play <FILE>`: play one audio file directly (does not use the library)
- `path <DIR>`: set the root music directory
- `refresh`: scan the configured root directory and rebuild the library
- `playlist`: play through the library (simple terminal UI)
- `list`: print the library as a list
- `search <QUERY>`: fuzzy search the library (title/artist/album)
- `select <INDEX>`: play one library entry by index (**0-based**, as printed by `search`)
- `sort [title|artist|album|duration]`: print the library sorted by a chosen field
- `browse`: open the interactive full-screen TUI browser/player
- `volume [0..100]`: set volume (or show current volume if omitted)
- `shuffle [true|false]`: toggle shuffle (or set it explicitly if provided)
- `loop [off|all|one]`: cycle repeat mode (or set it explicitly if provided)
- `status`: show the current player status
- `prev-threshold [5..100]`: set the “previous track restart” threshold (or show current if omitted)

### Examples

```bash
hextune list
hextune search "pink floyd wall"
hextune select 42

hextune volume 70
hextune shuffle true
hextune loop all
hextune prev-threshold 25

hextune sort artist
```

## `browse` (TUI) key bindings

### Normal mode

- Navigate: `↑/↓` or `j/k`
- Play selected: `Enter`
- Pause/resume: `Space` or `p`
- Next/previous: `n` / `b` (also `→` / `←`)  “previous” restarts the current track if played ≥ threshold%, otherwise skips back
- Toggle shuffle: `r`
- Cycle sort field: `o`
- Refresh library scan: `F5` or `u`
- Search: `/` or `Ctrl+f`
- Settings: `s`
- Quit: `q`, `Esc`, or `Ctrl+c`

### Search mode

- Type to search (fuzzy)
- Clear query: `Ctrl+u`
- Backspace: delete last character
- Navigate results: `↑/↓`
- Play selected: `Enter`
- Pause/resume: `Ctrl+Space`
- Exit search: `Esc`

### Settings modal

- Open/close: `s` (or close with `Esc`)
- Navigate fields: `↑/↓` or `j/k`
- **Volume**: `Enter` to edit, `←/→` adjusts by 5, digits type a value, `Enter` confirm, `Esc` cancel
- **Repeat**: `Enter`/`→` cycles forward, `←` cycles backward
- **Prev threshold**: `Enter` to edit, `←/→` adjusts by 5, digits type a value, `Enter` confirm, `Esc` cancel (5–100%)
- **Music path**: `Enter` to edit, type a path, `Enter` to confirm (validated), `Esc` cancel, `Ctrl+u` clear

## Keymap configuration (`keymap.toml`)

You can override the default TUI key bindings by editing:
  
- `<config dir>/hextune/keymap.toml`

If the file is missing, `hextune` creates it with the compiled-in defaults and uses
those defaults. If the file exists but is invalid TOML, `hextune` prints a warning to
stderr and falls back to defaults.

### Format

- Sections: `[normal]`, `[search]`, `[settings]`
- Value types: a string (single key) or an array of strings (multiple keys)
- Key strings look like: `q`, `Esc`, `Enter`, `Space`, `Ctrl+c`, `Ctrl+Space`, `F5`, `Up`

Example:

```toml
[normal]
quit = ["q", "Esc", "Ctrl+c"]
open_settings = "s"
enter_search = ["/", "Ctrl+f"]
navigate_up = ["Up", "k"]
navigate_down = ["Down", "j"]
play_selected = "Enter"
toggle_pause = ["Space", "p"]
next_track = ["n", "Right"]
prev_track = ["b", "Left"]
toggle_shuffle = "r"
refresh = ["F5", "u"]
cycle_sort = "o"

[search]
search_exit = "Esc"
toggle_pause = "Ctrl+Space"
clear_line = "Ctrl+u"
navigate_up = "Up"
navigate_down = "Down"
play_selected = "Enter"

[settings]
settings_close = ["Esc", "s"]
settings_confirm = "Enter"
settings_left = "Left"
settings_right = "Right"
clear_line = "Ctrl+u"
navigate_up = ["Up", "k"]
navigate_down = ["Down", "j"]

```

### Supported actions

`keymap.toml` can remap these action keys:

- `[normal]`: `quit`, `open_settings`, `enter_search`, `navigate_up`, `navigate_down`,
  `play_selected`, `toggle_pause`, `next_track`, `prev_track`, `toggle_shuffle`, `refresh`,
  `cycle_sort`
- `[search]`: `search_exit`, `toggle_pause`, `clear_line`, `navigate_up`, `navigate_down`,
  `play_selected`
- `[settings]`: `settings_close`, `settings_confirm`, `settings_left`, `settings_right`,
  `clear_line`, `navigate_up`, `navigate_down`

Text entry is intentionally not configurable:

- Search text input always types characters (and `Backspace` always deletes).
- When editing the settings path, character input always types (and `Backspace` always deletes).

## Data storage

`hextune` stores its state (library + settings like volume/shuffle/repeat/prev-threshold/path) in:

- `<config dir>/hextune/db.json`

The *config dir* is your OS config directory as reported by `dirs::config_dir()` (it differs
across platforms).

If the file becomes corrupted, `hextune` will try to recover what it can; otherwise it
backs it up as `db.json.bak` and starts with defaults.

## Supported audio files

Library scanning includes files with these extensions (case-insensitive):

- `mp3`, `flac`, `wav`, `ogg`

Metadata (title/artist/album/duration) is read when available; otherwise the filename is
used as the title.

## Development

```bash
cargo test
cargo run -- --help
```

## Troubleshooting

- “No music path set”: run `hextune path <DIR>` (or set it in `browse` → Settings → Music path)
- “Library is empty”: run `hextune refresh`
- `select` fails with “Invalid index …”: use `hextune search <QUERY>` to find the correct **0-based** index


## License
This project is licensed under the MIT License. See the [LICENSE](https://github.com/Condorbox/hextune/blob/main/LICENSE) file for more details.
