#!/bin/bash
#
# veracrypt-security-exit.sh
#
# Unmounts every currently mounted VeraCrypt volume. Installed by the
# veracrypt-security-exit package. Called by:
#   - veracrypt-security-exit.service (before shutdown/reboot)
#   - any desktop logout hook the user configures (see README)
#
set -euo pipefail

exec /usr/bin/veracrypt --text --dismount
