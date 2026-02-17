use crate::adapters::{portal::PortalAdapter, system::SystemAdapter};
use crate::diagnostics::{DiagnosticItem, Severity, TroubleshootReport};

pub fn run_troubleshooting(system: &SystemAdapter, portal: &PortalAdapter) -> TroubleshootReport {
    let mut items = Vec::new();

    for cmd in ["niri", "wpctl", "wl-mirror"] {
        let ok = system.command_exists(cmd);
        items.push(DiagnosticItem {
            title: format!("Command: {cmd}"),
            severity: if ok { Severity::Ok } else { Severity::Error },
            message: if ok {
                "available".to_string()
            } else {
                "missing".to_string()
            },
            remediation: format!("Install package that provides `{cmd}` on Arch Linux."),
        });
    }

    let frontend = portal.has_portal_frontend(system);
    items.push(DiagnosticItem {
        title: "Portal frontend".to_string(),
        severity: if frontend { Severity::Ok } else { Severity::Error },
        message: if frontend {
            "xdg-desktop-portal detected (service, PATH, or known system path)".to_string()
        } else {
            "xdg-desktop-portal not detected".to_string()
        },
        remediation: "Install xdg-desktop-portal and ensure xdg-desktop-portal.service can run as a user service.".to_string(),
    });

    for svc in [
        "pipewire.service",
        "wireplumber.service",
        "xdg-desktop-portal.service",
    ] {
        let active = system.service_is_active(svc);
        items.push(DiagnosticItem {
            title: format!("Service: {svc}"),
            severity: if active { Severity::Ok } else { Severity::Warn },
            message: if active {
                "active".to_string()
            } else {
                "inactive".to_string()
            },
            remediation: format!("Enable/start user service: `systemctl --user enable --now {svc}`."),
        });
    }

    let wayland = system.env_var("WAYLAND_DISPLAY");
    items.push(DiagnosticItem {
        title: "Env: WAYLAND_DISPLAY".to_string(),
        severity: if wayland.is_some() {
            Severity::Ok
        } else {
            Severity::Error
        },
        message: wayland.unwrap_or_else(|| "not set".to_string()),
        remediation: "Start niri as a proper Wayland session (display manager or niri-session).".to_string(),
    });

    let desktop = system.env_var("XDG_CURRENT_DESKTOP");
    items.push(DiagnosticItem {
        title: "Env: XDG_CURRENT_DESKTOP".to_string(),
        severity: if desktop.is_some() {
            Severity::Ok
        } else {
            Severity::Warn
        },
        message: desktop.unwrap_or_else(|| "not set".to_string()),
        remediation: "Import environment into systemd user session so portals see desktop vars.".to_string(),
    });

    let portal_ready = portal.has_gnome_portal_backend(system);
    items.push(DiagnosticItem {
        title: "Portal backend (GNOME)".to_string(),
        severity: if portal_ready {
            Severity::Ok
        } else {
            Severity::Error
        },
        message: if portal_ready {
            "xdg-desktop-portal-gnome detected (binary or portal metadata)".to_string()
        } else {
            "expected screencast portal backend not detected".to_string()
        },
        remediation: "Install/configure xdg-desktop-portal-gnome and restart user portal services; PATH is optional if backend exists in /usr/lib or /usr/libexec.".to_string(),
    });

    TroubleshootReport::new(items)
}
