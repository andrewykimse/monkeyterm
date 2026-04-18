# monkeyterm

A terminal typing test inspired by [MonkeyType](https://monkeytype.com), built with Rust and [Ratatui](https://ratatui.rs).

## Features

- **Multiple test modes** — words (25/50/100), timed (30s/60s/120s), and quote
- **Live stats** — real-time WPM, accuracy, and timer/progress during the test
- **Results screen** — final WPM, raw WPM, accuracy, and character breakdown
- **10 themes** — Serika Dark, Dracula, Nord, Catppuccin, Solarized Dark, Gruvbox, Monokai, One Dark, Tokyo Night, Rose Pine
- **Theme picker** with live preview

## Install

**With Nix flakes:**

```sh
nix profile install github:andrewkim/monkeyterm
```

Or run without installing:

```sh
nix run github:andrewkim/monkeyterm
```

**With Cargo:**

```sh
cargo install --path .
```

Or run directly:

```sh
cargo run --release
```

## Development

**With Nix:**

```sh
nix develop        # enter dev shell with cargo, rustc, rustfmt, clippy
nix build          # build release binary → ./result/bin/monkeyterm
```

**With Cargo:**

```sh
cargo build --release
```

## Usage

Launch `monkeyterm` and use keyboard shortcuts to navigate.

### Home screen

| Key | Action |
|-----|--------|
| `1` | Words 25 |
| `2` | Words 50 |
| `3` | Words 100 |
| `4` | Time 30s |
| `5` | Time 60s |
| `6` | Time 120s |
| `c` | Quote |
| `t` | Theme picker |
| `q` / `Ctrl+C` | Quit |

### Typing screen

| Key | Action |
|-----|--------|
| `Tab` / `Esc` | Return to home |
| `Backspace` | Delete last character |
| `Ctrl+Backspace` | Delete current word |
| `Space` | Commit word |

### Results screen

| Key | Action |
|-----|--------|
| `Tab` / `Enter` / `r` | Restart test |
| `Esc` / `q` | Return to home |

### Theme picker

| Key | Action |
|-----|--------|
| `j` / `↓` | Next theme |
| `k` / `↑` | Previous theme |
| `Enter` | Select theme |
| `Esc` / `q` | Cancel |

## Requirements

- Rust 1.85+ (edition 2024), or Nix with flakes enabled
- A terminal with true color support
