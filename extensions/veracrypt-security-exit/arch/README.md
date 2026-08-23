# veracrypt-security-exit

Arch Linux package that automatically unmounts all mounted VeraCrypt volumes
before shutdown/reboot, and provides a group-based sudoers rule so desktop
logout hooks can do the same without a password prompt.

## What's Installed

| Path | Purpose |
|---|---|
| `/usr/bin/veracrypt-security-exit.sh` | Runs `veracrypt --text --dismount` |
| `/usr/lib/systemd/system/veracrypt-security-exit.service` | Runs the script automatically before `shutdown.target`/`reboot.target` |
| `/etc/sudoers.d/veracrypt-security-exit` | Lets members of the `veracrypt-security-exit` group run the script passwordless |

No specific username is hardcoded into the package — access is controlled by
group membership instead, since that's the only approach that makes sense
for a redistributable package.

## Building

```bash
makepkg -si
```

This builds and installs the package in one step (`-s` resolves deps with
your AUR helper/pacman, `-i` installs the result).

If you plan to publish this to the AUR, generate real checksums first and
regenerate `.SRCINFO`:

```bash
updpkgsums
makepkg --printsrcinfo > .SRCINFO
```

## After Installing

1. Enable the shutdown/reboot hook (the package doesn't auto-enable it, per
   Arch packaging conventions):
   ```bash
   sudo systemctl enable veracrypt-security-exit.service
   ```

2. Add your user to the group so logout hooks can run the script without a
   password prompt:
   ```bash
   sudo usermod -aG veracrypt-security-exit "$USER"
   ```
   Log out and back in for the group membership to take effect.

Both steps are also printed automatically after install/upgrade.

## Testing

```bash
sudo systemctl stop veracrypt-security-exit.service
```

This triggers the same unmount the shutdown hook would run. With the group
membership from step 2 above, you should also be able to run this without
`sudo` prompting for a password:

```bash
sudo veracrypt-security-exit.sh
```

## Hooking Into Desktop Logout

The systemd service only covers shutdown/reboot. For logout (session ends,
machine stays on), wire it into your desktop environment:

**Universal (recommended when available) — `systemd --user`:**

```bash
mkdir -p ~/.config/systemd/user
cat > ~/.config/systemd/user/veracrypt-security-exit.service << 'EOF'
[Unit]
PartOf=graphical-session.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/true
ExecStop=/usr/bin/sudo /usr/bin/veracrypt-security-exit.sh

[Install]
WantedBy=graphical-session.target
EOF

systemctl --user daemon-reload
systemctl --user enable veracrypt-security-exit.service
```

Works out of the box on sessions tracked by `systemd --user` — including
Hyprland via `uwsm`, and most modern GNOME sessions. Check with
`systemctl --user status`.

**Hyprland (keybind alternative):**

```
bind = $mainMod SHIFT, Q, exec, sudo /usr/bin/veracrypt-security-exit.sh && hyprctl dispatch exit
```

**XFCE:** add a custom Application Shortcut (Settings → Keyboard →
Application Shortcuts) running:

```
bash -c "sudo /usr/bin/veracrypt-security-exit.sh; xfce4-session-logout --logout"
```

## Uninstalling

```bash
sudo pacman -R veracrypt-security-exit
```

The `.install` scriptlet disables the systemd service and removes the
`veracrypt-security-exit` group automatically. If you set up the
`systemd --user` logout hook manually, remove it yourself:

```bash
systemctl --user disable --now veracrypt-security-exit.service
rm ~/.config/systemd/user/veracrypt-security-exit.service
```
