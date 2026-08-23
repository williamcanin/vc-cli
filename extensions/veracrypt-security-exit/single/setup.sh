#!/bin/bash
#
# setup.sh
#
# Installs veracrypt-security-exit on any systemd distribution, without a
# package manager. Mirrors the exact layout used by the veracrypt-security-exit
# Arch Linux package:
#   1. /usr/bin/veracrypt-security-exit.sh                      unmount helper script
#   2. /usr/lib/systemd/system/veracrypt-security-exit.service  shutdown/reboot hook
#   3. /etc/sudoers.d/veracrypt-security-exit                   group-based passwordless rule
#   4. ~/.local/bin/hypr-logout.sh                              Hyprland logout convenience script
#
# Usage: sudo ./setup.sh
#

set -euo pipefail

SCRIPT_NAME="veracrypt-security-exit.sh"
SCRIPT_PATH="/usr/bin/${SCRIPT_NAME}"
GROUP_NAME="veracrypt-security-exit"
SUDOERS_PATH="/etc/sudoers.d/${GROUP_NAME}"
SERVICE_NAME="${GROUP_NAME}.service"
SERVICE_PATH="/usr/lib/systemd/system/${SERVICE_NAME}"

# --- 0. Initial checks ------------------------------------------------------

if [[ $EUID -ne 0 ]]; then
    echo "Error: run this script with sudo." >&2
    echo "  sudo ./setup.sh" >&2
    exit 1
fi

if ! command -v veracrypt &> /dev/null; then
    echo "Error: the 'veracrypt' binary was not found in PATH." >&2
    exit 1
fi

if ! command -v systemctl &> /dev/null; then
    echo "Error: systemd was not found (systemctl missing)." >&2
    exit 1
fi

# Detect the "real" user (the user who invoked sudo), not root
TARGET_USER="${SUDO_USER:-}"
if [[ -z "$TARGET_USER" ]]; then
    read -rp "Could not detect the user via sudo. Enter your Linux username: " TARGET_USER
fi

if ! id "$TARGET_USER" &> /dev/null; then
    echo "Error: user '$TARGET_USER' does not exist." >&2
    exit 1
fi

TARGET_HOME=$(getent passwd "$TARGET_USER" | cut -d: -f6)
VERACRYPT_BIN=$(command -v veracrypt)

echo "==> Target user: $TARGET_USER (home: $TARGET_HOME)"
echo "==> VeraCrypt binary: $VERACRYPT_BIN"
echo

# --- 1. Unmount script ------------------------------------------------------

echo "==> Creating $SCRIPT_PATH"
cat > "$SCRIPT_PATH" << EOF
#!/bin/bash
#
# ${SCRIPT_NAME}
#
# Unmounts every currently mounted VeraCrypt volume. Installed by setup.sh,
# mirroring the veracrypt-security-exit Arch Linux package. Called by:
#   - ${SERVICE_NAME} (before shutdown/reboot)
#   - any desktop logout hook the user configures (see README)
#
set -euo pipefail

exec ${VERACRYPT_BIN} --text --dismount
EOF
chmod 755 "$SCRIPT_PATH"
echo "    OK"
echo

# --- 2. Group and restricted sudoers rule ------------------------------------
# Same mechanism as the Arch package: access is controlled by membership of
# the 'veracrypt-security-exit' group, not by a hardcoded username.

echo "==> Creating system group '$GROUP_NAME'"
getent group "$GROUP_NAME" > /dev/null || groupadd --system "$GROUP_NAME"

if ! id -nG "$TARGET_USER" | tr ' ' '\n' | grep -qx "$GROUP_NAME"; then
    usermod -aG "$GROUP_NAME" "$TARGET_USER"
    echo "    Added '$TARGET_USER' to '$GROUP_NAME' (log out and back in for it to take effect)"
else
    echo "    '$TARGET_USER' is already a member"
fi
echo

echo "==> Creating $SUDOERS_PATH"
cat > "$SUDOERS_PATH" << EOF
# Installed by setup.sh (veracrypt-security-exit).
# Do not edit by hand — changes may be overwritten when setup.sh runs again.
#
# Members of the $GROUP_NAME group may run the unmount script
# without a sudo password prompt. This is what makes desktop logout hooks
# (Hyprland keybind, systemd --user unit, etc.) work without interrupting
# the logout flow with a password prompt.
#
# Add more users to the group with:
#   sudo usermod -aG $GROUP_NAME USERNAME
%${GROUP_NAME} ALL=(root) NOPASSWD: ${SCRIPT_PATH}
EOF
chmod 440 "$SUDOERS_PATH"

# Validate the syntax before trusting the file — prevents breaking system sudo
if ! visudo -c -f "$SUDOERS_PATH" &> /dev/null; then
    echo "Error: invalid syntax generated in $SUDOERS_PATH. Removing for safety." >&2
    rm -f "$SUDOERS_PATH"
    exit 1
fi
echo "    OK (syntax validated)"
echo

# --- 3. systemd service (shutdown/reboot) ------------------------------------

echo "==> Creating $SERVICE_PATH"
cat > "$SERVICE_PATH" << EOF
[Unit]
Description=Unmount all VeraCrypt volumes before shutdown/reboot
DefaultDependencies=no
Before=shutdown.target reboot.target halt.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/true
ExecStop=${SCRIPT_PATH}

[Install]
WantedBy=multi-user.target
EOF
chmod 644 "$SERVICE_PATH"

systemctl daemon-reload
echo "    OK (run 'systemctl enable ${SERVICE_NAME}' to activate the shutdown/reboot hook)"
echo

# --- 4. Hyprland logout script ----------------------------------------------

LOCAL_BIN="$TARGET_HOME/.local/bin"
LOGOUT_SCRIPT="$LOCAL_BIN/hypr-logout.sh"

echo "==> Creating $LOGOUT_SCRIPT"
mkdir -p "$LOCAL_BIN"
cat > "$LOGOUT_SCRIPT" << EOF
#!/bin/bash
# Unmounts VeraCrypt volumes and exits the Hyprland session.
sudo ${SCRIPT_PATH}
hyprctl dispatch exit
EOF
chmod 755 "$LOGOUT_SCRIPT"
chown "$TARGET_USER:$TARGET_USER" "$LOGOUT_SCRIPT"
chown "$TARGET_USER:$TARGET_USER" "$LOCAL_BIN" 2>/dev/null || true
echo "    OK"
echo

# --- Summary ---------------------------------------------------------------
# Mirrors the post-install instructions printed by the Arch package.

cat << EOF
==========================================================================
veracrypt-security-exit installed!

1. Enable the shutdown/reboot hook:
     systemctl enable ${SERVICE_NAME}

2. Group membership: '${TARGET_USER}' was added to '${GROUP_NAME}',
   so logout hooks can run the script without a sudo password prompt.
   Log out and back in for the group change to take effect.
   To add other users later:
     sudo usermod -aG ${GROUP_NAME} USERNAME

3. Hyprland logout: open your hyprland.conf and replace your current
   logout bind with:

       bind = \$mainMod SHIFT, Q, exec, $LOGOUT_SCRIPT

   (adjust the key combination to match the one you currently use)

4. If you use a power menu (wlogout, etc.), point its logout action to:
   $LOGOUT_SCRIPT

To test the shutdown/reboot hook after enabling:
     sudo systemctl stop ${SERVICE_NAME}

To manually test unmounting at any time:
     sudo ${SCRIPT_PATH}
==========================================================================
EOF
