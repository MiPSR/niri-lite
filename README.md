<h1 align="center">niri-lite</h1>
<p align="center">A lite fork of <a href="https://github.com/niri-wm/niri">niri</a>, a scrollable-tiling Wayland compositor.</p>

> [!WARNING]
> **Experimental project.** This fork exists to experiment with stripping niri down.
> It is not a good idea to use it: expect breakage, missing features, and no support.
> If you want a compositor that works, use [upstream niri](https://github.com/niri-wm/niri).
>
> An AI agent was used to help make these changes.

## About

Windows are arranged in columns on an infinite strip going to the right, with
dynamic per-monitor workspaces. See the
[upstream README](https://github.com/niri-wm/niri#about) and the
[documentation](https://niri-wm.github.io/niri/) for everything niri can do.

This fork tracks upstream and only diverges where noted below.

## Removed

- The CLI (`src/cli.rs`)
- The `niri msg` client (`src/ipc/client.rs`)
- clap and completion dependencies (`Cargo.toml`)

## Added

- Minimal hand-rolled parsing of the launch flags (`src/main.rs`):
  `-c/--config <path>`, `--session`, commands after `--`. Anything else is
  ignored with a warning.
- Renamed user-facing identity to niri-lite: binary (`Cargo.toml`), config
  directory `~/.config/niri-lite` (`src/main.rs`), desktop, session, service,
  portal files (`resources/`), IPC socket filename (`src/ipc/server.rs`).

## Development

A Nix shell with all build dependencies is provided:

```sh
nix-shell
cargo build
```

## Contributing

Please contribute to the original projects instead:

- [niri](https://github.com/niri-wm/niri) — see its
  [CONTRIBUTING.md](https://github.com/niri-wm/niri/blob/main/CONTRIBUTING.md)

Only open issues or pull requests here for changes that are specific to this
lite fork.

## License

GPL-3.0-or-later, same as upstream niri. See [LICENSE](LICENSE).
