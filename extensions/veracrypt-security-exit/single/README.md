# veracrypt-security-exit (single-script installer)

Automatically unmounts all mounted VeraCrypt volumes before shutdown/reboot,
and provides a ready-to-use logout script so you can unmount volumes cleanly
when you end your desktop session too.

This exists to prevent the classic `mount: wrong fs type, bad option, bad
superblock` error that happens when an NTFS-formatted VeraCrypt volume gets
left in a "dirty" state after an unclean unmount (e.g. shutting down or
logging out while it's still mounted).

This is the distro-agnostic version of the [Arch package](../arch/): `setup.sh`
installs the same files, with the same names and paths, on any systemd
distribution.

## What Gets Installed

| File | Purpose |
|---|---|
| `/usr/bin/veracrypt-security-exit.sh` | Runs `veracrypt --text --dismount` |
| `/usr/lib/systemd/system/veracrypt-security-exit.service` | Fires the unmount script automatically before `shutdown.target`/`reboot.target` |
| `/etc/sudoers.d/veracrypt-security-exit` | Lets members of the `veracrypt-security-exit` group run the unmount script with `sudo` and no password prompt (scoped to that one script only) |
| `~/.local/bin/hypr-logout.sh` | Convenience script: unmounts volumes, then exits the session |

Like the Arch package, access is controlled by membership of the
`veracrypt-security-exit` group — no username is hardcoded into any file.
`setup.sh` creates the system group and adds your user to it automatically.

## Requirements

- `veracrypt` installed and available in `PATH`
- `systemd`
- `sudo`

## Installation

```sh
chmod +x setup.sh
sudo ./setup.sh
```

The script detects your regular (non-root) username automatically via
`$SUDO_USER`. If that can't be determined, it will ask you to type it in.

After it finishes, enable the shutdown/reboot hook:

```sh
systemctl enable veracrypt-security-exit.service
```

Log out and back in so the group membership takes effect.

## Testing

Shutdown/reboot hook:

```sh
sudo systemctl stop veracrypt-security-exit.service
```

This should unmount every currently mounted VeraCrypt volume. If nothing is
mounted, it's a no-op.

Manual unmount at any time:

```sh
sudo /usr/bin/veracrypt-security-exit.sh
```

## Hooking Into Logout

The shutdown/reboot hook works everywhere automatically, no extra setup
needed. Logout is different: since the machine stays on, there's no single
system-wide event to hang the unmount off of, so each desktop environment
needs its own wiring.

### Universal method (recommended if available): systemd user service

If your session is tracked by `systemd --user` — which is the case for
Hyprland when launched through `uwsm`, and for most modern GNOME and KDE
sessions — this method is more reliable than a keybind, because it fires
whenever the graphical session actually ends, not just when you press a
specific key combo.

```sh
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

This relies on the sudoers rule installed by `setup.sh`, so your user must be
a member of the `veracrypt-security-exit` group — which it already is if you
ran the installer.

If you're not sure whether your session uses `systemd --user`, check with:

```bash
systemctl --user status
```

If that returns session info instead of an error, you're good to use this
method. Otherwise, fall back to the DE-specific instructions below.

### Hyprland

`setup.sh` already created `~/.local/bin/hypr-logout.sh`.
Add a keybind for it in `hyprland.conf`, replacing (or alongside) your
current logout bind:

```
bind = $mainMod SHIFT, Q, exec, ~/.local/bin/hypr-logout.sh
```

If you use a power menu (`wlogout`, etc.), point its logout action at
`~/.local/bin/hypr-logout.sh` instead of calling `hyprctl dispatch exit`
directly.

### GNOME

GNOME doesn't expose a simple config file for logout hooks, so the systemd
user service method above is the recommended path — GNOME sessions are
tracked by `systemd --user` by default on any reasonably recent
distribution, so `graphical-session.target` fires correctly on logout.

If you'd rather not rely on that, an alternative is a GNOME Shell Extension
that runs a command pre-logout, but that's considerably more setup for the
same result — the systemd service is simpler and does the job.

### XFCE

XFCE doesn't have a native pre-logout hook, but you can replace the logout
action with a custom command:

1. Open **Settings → Keyboard → Application Shortcuts**

2. Add a new shortcut with the command:

```
   bash -c "sudo /usr/bin/veracrypt-security-exit.sh; xfce4-session-logout --logout"
```
3. Assign it to whatever key combination you currently use for logout
   (check **Settings → Session and Startup** or your panel's logout button
   for the one currently in use, so you can replace it consistently)
If your XFCE session happens to run under `systemd --user` (some distros
enable this), the universal systemd method above works too — check with
`systemctl --user status` as described earlier.

## Uninstalling

```sh
sudo systemctl disable --now veracrypt-security-exit.service
sudo rm /usr/lib/systemd/system/veracrypt-security-exit.service
sudo rm /usr/bin/veracrypt-security-exit.sh
sudo rm /etc/sudoers.d/veracrypt-security-exit
rm ~/.local/bin/hypr-logout.sh
sudo systemctl daemon-reload
sudo groupdel veracrypt-security-exit

# if you set up the systemd --user method too:
systemctl --user disable --now veracrypt-security-exit.service
rm ~/.config/systemd/user/veracrypt-security-exit.service
systemctl --user daemon-reload
```

## Notes

- The sudoers rule is scoped to exactly one script path — it does not grant
  broader `sudo` access.
- `veracrypt --text --dismount` unmounts *all* currently mounted VeraCrypt
  volumes, not a specific one. If you only want to unmount a single volume,
  call `veracrypt -d <path>` manually instead.
