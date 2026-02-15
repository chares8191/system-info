# system-info

Small CLI that prints a JSON snapshot of this machine for troubleshooting and context.

## Usage

```sh
cargo run -p system-info
cargo run -p system-info -- --pretty
cargo run -p system-info -- --pretty --indent=2
```

The output is a single JSON object to stdout. Some fields are optional and will be `null`
or empty strings when the underlying command cannot run or returns no data.

## Output structure (top-level)

```json
{
  "uname": { ... },
  "user": { ... },
  "env": { ... },
  "dmi": { ... },
  "xdg": { ... },
  "cpu": { ... },
  "proc": { ... },
  "mkinitcpio": { ... },
  "x11": { ... },
  "pacman": { ... },
  "lsblk": [ ... ],
  "lspci": [ ... ],
  "lsmod": [ ... ]
}
```

Notes:
- `lsmod` is intentionally last because it is the largest section.
- `x11` subfields can be missing if no X display is available.

## Section details

### uname
- `kernel_release`, `machine`

### user
- `username`, `uid`, `gid`, `home_directory`, `login_shell`

### env
- Common environment variables: `user`, `logname`, `home`, `shell`, `path`, `lang`,
  `lc_all`, `lc_ctype`, `term`

### dmi
- BIOS and hardware identifiers from `/sys/class/dmi/id/*`

### xdg
- XDG environment variables such as `XDG_CACHE_HOME`, `XDG_RUNTIME_DIR`, `XDG_SESSION_TYPE`, etc.

### cpu
- Selected `lscpu` fields (architecture, model, cores, cache, virtualization, etc.)

### proc
- `cmdline`, `version`
- `meminfo.mem_total` from `/proc/meminfo`

### mkinitcpio
- `modules`, `hooks` from `/etc/mkinitcpio.conf`

### x11
- `xinput.devices`: parsed `xinput list`
- `xrandr.monitors`: parsed `xrandr --listmonitors`
- `xrdb.resources`: line list from `xrdb -query`
- `xdpyinfo.dimensions`, `xdpyinfo.resolution`

### pacman
- `explicit`: `pacman -Qe` output lines

### lsblk
- One entry per `lsblk --json --list` row with filesystem metadata.
- Fields are stored as strings (including numbers) for simplicity.

### lspci
- One entry per `lspci -nnk` device with parsed IDs, subsystem info, and kernel modules.

### lsmod
- One entry per `lsmod` line: `module`, `size`, `used_by_count`, `used_by`

