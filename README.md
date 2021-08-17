[![pytest](https://github.com/steven-omaha/pacdef/actions/workflows/pytest.yml/badge.svg)](https://github.com/steven-omaha/pacdef/actions/workflows/pytest.yml)
[![pep257](https://github.com/steven-omaha/pacdef/actions/workflows/pep257.yml/badge.svg)](https://github.com/steven-omaha/pacdef/actions/workflows/pep257.yml)
[![bandit](https://github.com/steven-omaha/pacdef/actions/workflows/bandit.yml/badge.svg)](https://github.com/steven-omaha/pacdef/actions/workflows/bandit.yml)
[![black](https://github.com/steven-omaha/pacdef/actions/workflows/black.yml/badge.svg)](https://github.com/steven-omaha/pacdef/actions/workflows/black.yml)

# pacdef
declarative manager of Arch packages

## Installation
`pacdef` is available in the AUR [as stable release](https://aur.archlinux.org/packages/pacdef) or [development version](https://aur.archlinux.org/packages/pacdef-git).

## Use-case
`pacdef` allows the user to have consistent packages among multiple Arch machines by managing packages in group files.
The group files are maintained outside of `pacdef` by any VCS.

`pacdef import`ing a package group file creates a symlink to that file in `pacdef`'s config dir, thereby letting
`pacdef` know you want to have the packages in this file on your machine.
Then installing all packages from all groups is as simple as `pacdef sync`.
All package operations are executed by your favourite AUR helper.

If you work with multiple Arch installations and have asked yourself "*Why do I have the program that I use every day on
my other machine not installed here?*", then `pacdef` is the tool for you.

### Example
This tree shows my pacdef repository (not the `pacdef` config dir).
```
.
├── generic
│   ├── audio
│   ├── base
│   ├── desktop
│   ├── private
│   ├── rust
│   ├── wayland
│   ├── wireless
│   ├── work
│   └── xorg
├── hosts
│   ├── hostname_a
│   ├── hostname_b
│   └── hostname_c
└── pacdef.conf
```

* The `base` group holds all packages I need unconditionally, and includes things like zfs,
  [paru](https://github.com/Morganamilo/paru) and [neovim](https://github.com/neovim/neovim).
* In `xorg` and `wayland` I have stored the respective graphic servers and DEs.
* `wireless` contains tools like `iwd` and `bluez-utils` for machines with wireless interfaces.
* Under `hosts` I have one file for each machine I use. The filenames match the corresponding hostname. The packages
  are specific to one machine only, like device drivers, or any programs I use exclusively on that machine.

Usage on different machines: 
* home server: `base private hostname_a`
* private PC: `audio base desktop private rust wayland hostname_b`
* work PC: `base desktop rust work xorg hostname_c`

## How to use
* import one or more groups: `pacdef import base desktop audio`
* install packages from the imported groups: `pacdef sync`
* show packages that are not part of any group: `pacdef unmanaged`
* remove packages that are not in any group: `pacdef clean`
* show imported groups: `pacdef groups`
* remove a previously imported group: `pacdef remove audio`
* search for the group that contains a package: `pacdef search firefox`
* show packages of a group: `pacdef show desktop`


### Configuration

On first execution, it will create a basic config file under `$XDG_CONFIG_HOME/pacdef/pacdef.conf`
```ini
[misc]
aur_helper = paru
```

If you would like to use another AUR helper, you will need to change the config.

### package group files
1. One package per line. 
2. Anything after a `#` is ignored.
3. Empty lines are ignored.
4. If a package exists in multiple repositories, the repo can be specified as prefix followed by a forward slash.
   The AUR helper must understand this notation.

Example:
```ini
# desktop
alacritty
firefox
libreoffice-fresh
mycustomrepo/zsh-theme-powerlevel10k
...
```
