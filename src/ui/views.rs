use ratatui::text::{Line, Text};

use crate::app::App;

pub fn main_content(app: &App) -> Text<'static> {
    match app.selected_tab {
        0 => cast_view(app),
        1 => monitors_view(app),
        2 => audio_view(app),
        3 => profiles_view(app),
        _ => troubleshoot_view(app),
    }
}

fn cast_view(app: &App) -> Text<'static> {
    let mut lines = vec![
        Line::from("Cast preflight checks for PipeWire + xdg-desktop-portal + niri session."),
        Line::from("Press 'c' to run cast preflight checks."),
        Line::from("Press 'e' for extend-right mode."),
        Line::from("Press 'w' for extend-left mode."),
        Line::from("Press 'v' for mirror mode (wl-mirror fullscreen on HDMI)."),
        Line::from("Press 'h' for HDMI-only mode."),
        Line::from("Press 'u' to restore all connected outputs (turn on + auto position)."),
        Line::from(""),
    ];

    if let Some(report) = &app.diagnostics {
        lines.push(Line::from(format!(
            "Latest diagnostics: {} ok / {} warn / {} error",
            report.ok_count(),
            report.warn_count(),
            report.error_count()
        )));
    }

    Text::from(lines)
}

fn monitors_view(app: &App) -> Text<'static> {
    let mut lines = vec![
        Line::from("Monitor control via `niri msg` wrappers."),
        Line::from("Press 'm' to list HDMI outputs."),
        Line::from(""),
        Line::from("Discovered outputs:"),
    ];
    if app.last_outputs.is_empty() {
        lines.push(Line::from("- none"));
    } else {
        lines.extend(app.last_outputs.iter().map(|x| Line::from(format!("- {x}"))));
    }
    Text::from(lines)
}

fn audio_view(app: &App) -> Text<'static> {
    let mut lines = vec![
        Line::from("Audio control via `wpctl` wrappers."),
        Line::from("Press 'a' to switch to first HDMI sink (TV quick switch)."),
        Line::from("Press 't' for TV audio, 'p' for laptop audio quick switch."),
        Line::from("Use 'j'/'k' to select a sink, Enter to apply selected sink."),
        Line::from(""),
        Line::from("Discovered audio output channels:"),
    ];
    if app.audio_sinks.is_empty() {
        lines.push(Line::from("- none"));
    } else {
        for (idx, sink) in app.audio_sinks.iter().enumerate() {
            let cursor = if idx == app.selected_audio_sink { ">" } else { " " };
            let default = if sink.is_default { "*" } else { " " };
            let hdmi = if sink.is_hdmi { "TV/HDMI" } else { "Laptop/Analog" };
            lines.push(Line::from(format!(
                "{cursor} [{default}] {}. {} ({hdmi})",
                sink.id, sink.name
            )));
        }
    }
    Text::from(lines)
}

fn profiles_view(_app: &App) -> Text<'static> {
    Text::from(vec![
        Line::from("Profile actions:"),
        Line::from("- 's': save profile as default-tv"),
        Line::from("- 'l': load profile default-tv"),
        Line::from(""),
        Line::from("Profiles are stored under XDG config directory."),
    ])
}

fn troubleshoot_view(app: &App) -> Text<'static> {
    let mut lines = vec![
        Line::from("Troubleshooting checks:"),
        Line::from("- pipewire / wireplumber / xdg-desktop-portal services"),
        Line::from("- xdg-desktop-portal-gnome availability"),
        Line::from("- WAYLAND_DISPLAY and XDG_CURRENT_DESKTOP"),
        Line::from(""),
        Line::from("Press 'd' to run diagnostics."),
        Line::from(""),
    ];

    if let Some(report) = &app.diagnostics {
        lines.push(Line::from("Latest report:"));
        lines.extend(report.items.iter().map(|item| {
            Line::from(format!(
                "- [{}] {}: {}",
                item.severity.as_str(),
                item.title,
                item.message
            ))
        }));
    }

    Text::from(lines)
}
