# desktools

A small, flat-styled GUI for creating and editing Linux `.desktop` launchers.

Built with Rust and [iced](https://iced.rs).

## Features

- Home window listing every launcher created with desktools.
- Click a card to edit that launcher in its own window, or hit **New launcher**
  to open a blank editor window.
- Form for the usual fields: name, command, icon, working directory, comment,
  generic name, categories, keywords and startup WM class.
- Behavior flags: launch in a terminal, hide from menus, startup notification.
- Live preview of the generated `.desktop` file.
- **Pin to desktop** copies the launcher to your desktop and makes it executable.
- Settings for the applications/desktop directories, listing behavior, and a
  **System / Light / Dark** theme.
- Only launchers saved by desktools are listed by default (they carry
  `X-Desktools=true`); pre-existing `.desktop` files can be shown from Settings.

## Requirements

- Linux (X11 or Wayland)
- Rust 1.88 or newer (edition 2024)

## Build and run

```sh
make            # debug build
make release    # optimized build
make run        # build and launch
make test       # run the test suite
make clippy     # lint
```

Or with Cargo directly:

```sh
cargo run --release
```

## Install

```sh
sudo make install               # /usr/local
sudo make install PREFIX=/usr   # /usr
sudo make uninstall
```

`install` places the binary in `$(PREFIX)/bin`, a desktop entry in
`$(PREFIX)/share/applications`, and the license in
`$(PREFIX)/share/licenses/desktools`. `DESTDIR` is supported for packaging.

## Arch Linux

`make arch` builds a pacman package using the `PKGBUILD` in `packaging/arch/`
and leaves the `.pkg.tar.zst` in `dist/`.

```sh
make arch
sudo pacman -U dist/desktools-*.pkg.tar.zst
```

Requires `base-devel` (for `makepkg`) and network access for the Rust crates.
The package is built from a local source tarball produced by `make dist`, so it
works before the project has any public release.

## Usage

1. Launch `desktools`.
2. Hit **New launcher**, fill in at least a name and a command, then **Save**.
3. The launcher appears in the home grid. Click it any time to edit it.
4. Use **Pin to desktop** to drop an executable copy on your desktop.

Saved launchers go to your applications directory, by default
`~/.local/share/applications`. Change it in **Settings**.

### Settings

Settings are stored in `~/.config/desktools/config` (or under
`$XDG_CONFIG_HOME`):

```ini
applications_dir=/home/me/.local/share/applications
desktop_dir=/home/me/Desktop
show_all=false
theme=system
```

- `theme`: `system`, `light` or `dark`. `system` follows the desktop preference.
- `show_all`: show all `.desktop` files, not only the ones made with desktools.

### Window behavior

Closing any window exits the application.

## License

MIT. See [LICENSE](LICENSE).
