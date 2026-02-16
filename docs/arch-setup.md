# Arch Linux setup notes

Install required packages:

```bash
sudo pacman -S --needed niri pipewire wireplumber xdg-desktop-portal xdg-desktop-portal-gnome
```

Ensure user services are active:

```bash
systemctl --user enable --now pipewire.service wireplumber.service xdg-desktop-portal.service
```

If screencast still fails, restart portal services and relogin to niri session.
