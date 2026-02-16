use crate::adapters::{portal::PortalAdapter, system::SystemAdapter};

#[derive(Debug)]
pub struct CastPreflight {
    pub ready: bool,
    pub missing_items: Vec<String>,
}

impl CastPreflight {
    pub fn summary_line(&self) -> String {
        if self.ready {
            "cast preflight: ready".to_string()
        } else {
            format!("cast preflight: not ready ({} issue(s))", self.missing_items.len())
        }
    }
}

pub fn preflight(system: &SystemAdapter, portal: &PortalAdapter) -> CastPreflight {
    let mut missing = Vec::new();

    for cmd in ["niri", "wpctl"] {
        if !system.command_exists(cmd) {
            missing.push(format!("missing command: {cmd}"));
        }
    }

    if !portal.has_portal_frontend(system) {
        missing.push("portal frontend not detected".to_string());
    }
    if !portal.has_gnome_portal_backend(system) {
        missing.push("portal gnome backend not detected".to_string());
    }

    for svc in [
        "pipewire.service",
        "wireplumber.service",
        "xdg-desktop-portal.service",
    ] {
        if !system.service_is_active(svc) {
            missing.push(format!("user service not active: {svc}"));
        }
    }

    if system.env_var("WAYLAND_DISPLAY").is_none() {
        missing.push("WAYLAND_DISPLAY is not set".to_string());
    }
    if system.env_var("XDG_CURRENT_DESKTOP").is_none() {
        missing.push("XDG_CURRENT_DESKTOP is not set".to_string());
    }
    CastPreflight {
        ready: missing.is_empty() && portal.screencast_stack_ready(system),
        missing_items: missing,
    }
}
