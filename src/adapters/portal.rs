use crate::adapters::system::SystemAdapter;

#[derive(Debug, Default)]
pub struct PortalAdapter;

impl PortalAdapter {
    pub fn has_portal_frontend(&self, system: &SystemAdapter) -> bool {
        system.command_exists("xdg-desktop-portal")
            || system.service_is_active("xdg-desktop-portal.service")
            || system.any_path_exists(&[
                "/usr/lib/xdg-desktop-portal",
                "/usr/libexec/xdg-desktop-portal",
                "/usr/bin/xdg-desktop-portal",
            ])
    }

    pub fn has_gnome_portal_backend(&self, system: &SystemAdapter) -> bool {
        self.has_portal_frontend(system)
            && (system.command_exists("xdg-desktop-portal-gnome")
                || system.any_path_exists(&[
                    "/usr/lib/xdg-desktop-portal-gnome",
                    "/usr/libexec/xdg-desktop-portal-gnome",
                    "/usr/bin/xdg-desktop-portal-gnome",
                ])
                || system.any_path_exists(&[
                    "/usr/share/xdg-desktop-portal/portals/gnome.portal",
                    "/usr/share/xdg-desktop-portal/gnome-portals.conf",
                ]))
    }

    pub fn screencast_stack_ready(&self, system: &SystemAdapter) -> bool {
        system.service_is_active("pipewire.service")
            && system.service_is_active("wireplumber.service")
            && system.service_is_active("xdg-desktop-portal.service")
            && self.has_portal_frontend(system)
            && self.has_gnome_portal_backend(system)
    }
}
