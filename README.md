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

Open the configured file manager:

```sh
vc open
```
