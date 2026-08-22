# vc-cli

`vc-cli` is a small interactive command-line helper for VeraCrypt. It wraps the VeraCrypt CLI with friendlier commands for mounting, dismounting, listing, and opening volumes.

The package installs two commands:

- `vc-cli`
- `vc`

## Requirements

- Rust and Cargo
- GNU Make
- VeraCrypt CLI available as `veracrypt`
- `sudo`, because VeraCrypt mount and unmount operations are executed through `sudo veracrypt`
- Optional: `nautilus`, `spf`/Superfile, or another file manager
- Optional: `vim` or `nano` for `vc config open` when `$EDITOR` is not set

## Build

Run the full project check:

```sh
make check
```

Build the debug binaries:

```sh
make build
```

Build the release binaries:

```sh
make release
```

Reinstall the current local version:

```sh
make update
```

## Install

Install both `vc-cli` and `vc` using Cargo:

```sh
make install
```

Uninstall them:

```sh
make uninstall
```

## Configuration

The configuration file is optional. If it does not exist, `vc-cli` uses the system's default file picker or opener.

Create the default configuration:

```sh
vc config create
```

Open it in a terminal editor:

```sh
vc config open
```

Default configuration path:

```text
~/.config/vc-cli/config.toml
```

Default contents:

```toml
[general]
file_manager = "nautilus"
```

For Superfile, use:

```toml
[general]
file_manager = "spf"
```

## Usage

Mount a volume:

```sh
vc mount ~/vault.hc
```

Dismount one or more mounted slots:

```sh
vc umount
```

List mounted volumes:

```sh
vc list
```

Open a mounted volume in the configured file manager:

```sh
vc open
```

## Repair

Recover volumes with NTFS errors (bad sectors, dirty flag) in four steps:

```sh
vc fix mount ~/vault.hc   # 1. open without mounting the filesystem (interactive if no path)
vc fix list               # 2. shows the mapped device, e.g. /dev/mapper/veracrypt1
vc fix ntfs               # 3. runs ntfsfix and mounts at /tmp/veracrypt1 with ntfs3
vc fix umount             # 4. unmounts and dismounts all repair-mode volumes
```

`vc fix mount` accepts `-k/--keyfile <path>`; when omitted it asks interactively. The password is always requested by VeraCrypt directly, never passed as an argument.

`vc fix ntfs [MAPPER]` accepts a device path (`/dev/mapper/veracrypt1`), a device name (`veracrypt1`), or a slot number (`1`); interactive selection otherwise. Options: `-m/--mount-point <dir>` overrides `/tmp/<device>`, `--no-mount` skips mounting.

# Extras

Install the `veracrypt-security-exit` package for Arch Linux from the [extension](extension/) directory.

This package is essential for safely unmounting **VeraCrypt** drives upon logout, reboot, or system shutdown. This prevents errors, particularly with devices using the **NTFS** file system.
