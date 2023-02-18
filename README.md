# pacdef

multi-backend declarative package manager for Linux


## Installation

`pacdef` is available in the AUR [as stable release](https://aur.archlinux.org/packages/pacdef) or [development version](https://aur.archlinux.org/packages/pacdef-git), and on [crates.io](https://crates.io/crates/pacdef).

The AUR package will also provide completions for `zsh`.
If you use the `crates.io` version you will need to copy the completion file to the right directory yourself.


## Use-case

`pacdef` allows the user to have consistent packages among multiple Linux machines and different backends by managing packages in group files.
The idea is that (1) any package in the group files ("managed packages") will be installed explicitly, and (2) explicitly installed packages *not* found in any of the group files ("unmanaged packages") will be removed.
The group files are maintained outside of `pacdef` by any VCS, like git. 

If you work with multiple Linux machines and have asked yourself "*Why do I have the program that I use every day on my other machine not installed here?*", then `pacdef` is the tool for you.


## Of groups, sections, and packages

`pacdef` manages multiple package groups (group files) that, e.g., may be tied to a specific use-case.
Each group has one or more section(s) which correspond to a specific backend, like your system's package manager (`pacman`, `apt`, ...), or your programming languages package manger (`cargo`, `pip`, ...).
Each section contains one or more packages that can be installed respective package manager.


### Example

Let's assume you have the following group files.

`base`:

```ini
[arch]
paru
zsh

[rust]
pacdef
topgrade
```

`development`:

```ini
[arch]
rustup
rust-analyzer

[rust]
cargo-tree
flamegraph
```

Pacdef will make sure you have the following packages installed for each package manager:

- Arch (`pacman`, AUR helpers): paru, zsh, rustup, rust-analyzer
- Rust (`cargo`): pacdef, topgrade, cargo-tree, flamegraph

Note that the name of the section corresponds to the ecosystem it relates to, rather than the package manager it uses.


## Supported backends

At the moment, supported backends are limited to the following.

| Package Manager | Section   | Application |  Notes                                               |
|-----------------|-----------|-------------|------------------------------------------------------|
| `pacman`        | `arch`    | Arch Linux  |  includes pacman-wrapping AUR helpers (configurable) |
| `cargo`         | `rust`    | Rust        |                                                      |

Pull requests for additional backends are welcome!


### Example

This tree shows my pacdef repository (not the `pacdef` config dir).
```
.
├── generic
│   ├── audio
│   ├── base
│   ├── desktop
│   ├── private
│   ├── rust
│   ├── wayland
│   ├── wireless
│   ├── work
│   └── xorg
├── hosts
│   ├── hostname_a
│   ├── hostname_b
│   └── hostname_c
└── pacdef.conf
```

- The `base` group holds all packages I need unconditionally, and includes things like zfs,
  [paru](https://github.com/Morganamilo/paru) and [neovim](https://github.com/neovim/neovim).
- In `xorg` and `wayland` I have stored the respective graphic servers and DEs.
- `wireless` contains tools like `iwd` and `bluez-utils` for machines with wireless interfaces.
- Under `hosts` I have one file for each machine I use. The filenames match the corresponding hostname. The packages
  are specific to one machine only, like device drivers, or any programs I use exclusively on that machine.

Usage on different machines: 

- home server: `base private hostname_a`
- private PC: `audio base desktop private rust wayland hostname_b`
- work PC: `base desktop rust work xorg hostname_c`


## Commands

| Subcommand                        | Description                                                           |
|-----------------------------------|-----------------------------------------------------------------------|
| `group import [<group>...]`       | import one or more groups, which creates managed packages             |
| `group remove [<group>...]`       | remove a previously imported group                                    |
| `group new [-e] [<group>...]`     | create new groups, use `-e` to edit them immediately after creation   | 
| `group show [<group>...]`         | show contents of a group                                              |  
| `group list`                      | list names of all groups                                              |  
| `package sync`                    | install managed packages                                              |
| `package unmanaged`               | show all unmanaged packages                                           |
| `package clean`                   | remove all unmanaged packages                                         |
| `package review`                  | for each unmanaged package interactively decide what to do            |
| `package search <regex>`          | search for managed packages that match the search string              |
| `version`                         | show version information                                              |


## Configuration

On first execution, it will create a basic config file under `$XDG_CONFIG_HOME/pacdef/pacdef.yaml`.

```yaml
aur_helper: paru  # AUR helper to use on Arch Linux (paru, yay, ...)
aur_rm_args: null  # additional args to pass to AUR helper when removing packages (optional)
warn_not_symlinks: true  # warn if a group file is not a symlink
```


## Group file syntax

Group files loosely follow the syntax for `ini`-files.

1. Sections begin by their name in brackets.
2. One package per line. 
3. Anything after a `#` is ignored.
4. Empty lines are ignored.
5. If a package exists in multiple repositories, the repo can be specified as prefix followed by a forward slash.
   The package manager must understand this notation.

Example:
```ini
[pacman]
alacritty
firefox  # this comment is ignored
libreoffice-fresh
mycustomrepo/zsh-theme-powerlevel10k

[rust]
cargo-update
topgrade
```


## Naming

`pacdef` combines the words "package" and "define".


## supported rust version

latest stable
