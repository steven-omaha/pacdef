FROM archlinux:latest

RUN pacman -Syyu --noconfirm base-devel rust git zsh && \
    mkdir -p /root/.config/pacdef/groups && \
    git clone https://github.com/steven-omaha/pacdef && \
    cd pacdef && \
    cargo build -F arch && \
    echo $'aur_helper = "pacman"\nwarn_not_symlinks = false' > /root/.config/pacdef/pacdef.toml && \
    echo $'[arch]\ncore/base\nlinux' > /root/.config/pacdef/groups/my_group

WORKDIR /pacdef/target/debug

COPY entrypoint.zsh sync unmanaged /pacdef/target/debug/

ENTRYPOINT ["/usr/bin/zsh", "entrypoint.zsh"]

