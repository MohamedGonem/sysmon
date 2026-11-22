# Dependencies

This file records the current dependencies and why they are used.

## `libc`

Used instead of a higher-level crate such as `sysinfo`, because most data is
read directly from `/proc` and `/sys` and only a few syscalls are needed that
`std` does not expose:

- `rsysmon-core`: `sysconf` for clock ticks and page size, `statvfs` for disk
  space, `getifaddrs`/`freeifaddrs` for interface addresses, and `localtime_r`
  for boot-time formatting.
- `rsysmon-cli`: `signal` to restore the default `SIGPIPE` disposition, so
  piping output into `head` exits normally instead of panicking.

`libc` and the `/proc`/`/sys` collectors are Linux-specific. Platform-specific
collection should stay isolated so the core can gain Windows and macOS backends
later without changing the shared data model.

## `iced` (planned)

Implementing a GUI from scratch is time-consuming, so `iced` will be used for
the desktop GUI. It is not a dependency of the core. The core is intentionally
kept small so the GUI can depend on it without pulling in the GUI stack.

# Dependencies that will not be added

- `clap`: the custom argument parser keeps the release binary roughly 40%
  smaller than the `clap` version.
- `ratatui`: a TUI is not implemented yet; `crossterm` or manual ANSI output
  would be enough if one is added.
- `regex`: process search uses the small in-tree engine in
  `crates/core/src/regex.rs` (literals, `.`, `*`, `+`, `?`, `{m,n}`, classes,
  groups, alternation, anchors, and common escapes). It is pure Rust, so it
  works on every platform the core is ported to and avoids another dependency.
- `sysinfo`: the project needs raw Linux data that `sysinfo` does not expose
  (per-core governors, hwmon labels, mount flags, process file descriptors),
  and direct `/proc`/`/sys` reads keep the core small and transparent.
