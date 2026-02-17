# Usage

Start the TUI and use the built-in tabs:

1. **Cast**: run preflight checks.
2. **Monitors**: inspect HDMI outputs.
3. **Audio**: browse and switch all output channels.
4. **Profiles**: save/load reusable TV profile.
5. **Troubleshoot**: run complete diagnostics.

Cast controls:

- `e`: extend-right (place HDMI to the right)
- `w`: extend-left (place HDMI to the left)
- `v`: mirror via `wl-mirror` (mirror non-HDMI source fullscreen on HDMI)
- `h`: HDMI-only
- `u`: restore all outputs (on + auto position)

Audio controls:

- `j` / `k`: move selection across all detected output channels
- `Enter`: set selected channel as default
- `t`: quick switch to TV/HDMI audio (may switch card profile if HDMI sink is hidden)
- `p`: quick switch to laptop/non-HDMI audio (may switch card profile if analog sink is hidden)

Profiles are stored in:

- `$XDG_CONFIG_HOME/niri-cast/profiles.json`
- fallback usually `~/.config/niri-cast/profiles.json`
