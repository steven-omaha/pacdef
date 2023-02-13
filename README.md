# pacdef_rust

## output formats

### search

TODO: make these consistent!

```
base
----
[pacman]
base-devel

fwdh95
------
[pacman]
udev-led-rules

xorg
----
[pacman]
devour
```

### unmanaged
```
[pacman]
  libxcrypt-compat
  strace
```

### clean

```
Would remove the following packages
  [pacman]
    libxcrypt-compat
    strace
Continue? [Y/n] 
```

### review

```
pacman: libxcrypt-compat
assign to (g)roup, (d)elete, (s)kip, (i)nfo, (a)s dependency, (q)uit? d
pacman: strace
assign to (g)roup, (d)elete, (s)kip, (i)nfo, (a)s dependency, (q)uit? g
0: base
1: desktop
2: fwdh95
3: notebook
4: panthera
5: python
6: rust
7: ssd
8: wayland
9: wireless
10: work
11: xorg
0
[pacman]
delete:
  libxcrypt-compat
assign groups:
  strace -> base
Continue? [Y/n]
```

### sync
```
Would install the following packages:
  [pacman]
    broot
Continue? [Y/n]
```

### show

```
[pacman]
asciinema
autoconf
automake
base
base-devel
bashtop
bat
bat-extras
binutils
bison
broot
cheat
ctags
docker
docker-compose
dog
downgrade
duf
dust
fakeroot
fd
file
findpkg
findutils
flex
fzf
gawk
gcc
gettext
git
git-delta
gptfdisk
grep
groff
gzip
htop
hyperfine
iftop
inetutils
intel-ucode
iotop
kernel-modules-hook
libtool
linux
linux-firmware
linux-headers
linux-lts
linux-lts-headers
logrotate
lsb-release
lsd
lshw
lsix-git
lsof
m4
make
man-db
man-pages
moreutils
namcap
ncdu
neovim
neovim-symlinks
net-tools
nextcloud-client
nnn
ntp
openssh
overdue
pacman
pacman-contrib
pandoc-bin
paru-bin
patch
pkgconf
plocate
polkit
prettier
procps-ng
psmisc
pwgen
python
python-joblib
python-pydantic
python-pynvim
ripgrep
rsync
sd
sed
smartmontools
sshfs
sudo
texinfo
tmux
usbutils
viddy
vifm
vim-pacmanlog
vv-sixel-git
wget
which
yarn
yt-dlp
zfs-auto-snapshot
zfs-linux
zfs-linux-lts
zfs-undelete-git
zfs-utils
zoxide
zpool-scrub-unit
zsh
zsh-autosuggestions
zsh-completions
zsh-syntax-highlighting
community/zsh-theme-powerlevel10k
zsh-you-should-use

[rust]
cargo-update
pacdef
topgrade
```

