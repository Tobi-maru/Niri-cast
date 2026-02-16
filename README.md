# niri-cast

`niri-cast` is a Rust TUI for Arch Linux + niri that unifies:

- TV screencast preflight checks (PipeWire + portals)
- HDMI monitor discovery/control entrypoints
- HDMI audio sink switching
- profile save/load for repeated setups
- troubleshooting with actionable remediation

## Current status

Initial implementation scaffold is complete and runnable.

## Build

```bash
cargo build
```

## Run

```bash
cargo run
```

## Key bindings

- `q`: quit
- `Tab` / `Shift+Tab`: switch tabs
- `r`: refresh output + sink discovery
- `d`: run diagnostics
- `c`: cast preflight
- `e`: cast mode extend-right
- `w`: cast mode extend-left
- `v`: cast mode mirror
- `h`: cast mode HDMI-only
- `u`: restore all connected outputs (turn on + auto position)
- `m`: list HDMI outputs
- `a`: switch to first HDMI sink
- `s`: save profile (`default-tv`)
- `l`: load profile (`default-tv`)

## Runtime dependencies (Arch)

- `niri`
- `pipewire`
- `wireplumber`
- `xdg-desktop-portal`
- `xdg-desktop-portal-gnome`
- `wpctl` (from PipeWire stack)

See [docs/arch-setup.md](docs/arch-setup.md).
