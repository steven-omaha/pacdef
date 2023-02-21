#compdef pacdef

_pacdef() {
    integer ret=1
    local line

    if [ -z "${XDG_CONFIG_HOME}" ]; then
        GROUPDIR="~/.config/pacdef/groups"
    else
        GROUPDIR="${XDG_CONFIG_HOME}/pacdef/groups"
    fi

    function _subcommands {
        local -a subcommands
        subcommands=(
            'group:manage groups (alias: g)'
            'package:manage packages (alias: p)'
            'version:show version info (alias: v)'
        )
        _describe 'subcommand' subcommands
    }

    function _group_actions {
        local -a group_actions
        group_actions=(
            'edit:edit an imported group file (alias: e)'
            'list:show names of imported groups (alias: l)'
            'import:import a new group file (alias: i)'
            'new:create a new group file (alias: n)'
            'remove:remove a group file (alias: r)'
            'show:show packages under an imported group (alias: s)'
        )
        _describe 'group action' group_actions
    }


    function _package_actions {
        local -a package_actions
        package_actions=(
            'clean:uninstall packages not managed by pacdef (alias: c)'
            'review:review unmanaged packages (alias: r)'
            'search:show the group containing a package (alias: se)'
            'sync:install all packages from imported groups (alias: sy)'
            'unmanaged:show explicitly installed packages not managed by pacdef (alias: u)'
        )
        _describe 'package action' package_actions
    }

    _arguments -C \
        "1: :_subcommands" \
        "*::arg:->args" \
        && ret=0

    case $state in
        (args)
            case $line[1] in
                package)
                    case $line[2] in
                        search)
                            _arguments \
                                "2:regex:" && ret=0
                        ;;
                        (clean|review|sync|unmanaged|help)
                            _message "no more arguments" && ret=0
                        ;;
                        *)
                            _arguments \
                                "1: :_package_actions" \
                                "*::arg:->args" && ret=0
                        ;;
                    esac
                ;;
                group)
                    case $line[2] in
                        list)
                            _message "no more arguments" && ret=0
                        ;;
                        (edit|remove|show)
                            _arguments "*:group file:_files -W '$GROUPDIR'" && ret=0

                        ;;
                        import)
                            _arguments "*:new group file(s):_files" && ret=0
                        ;;
                        new)
                            _arguments \
                                {-e,--edit}"[edit group file after creating them]" \
                                "*:new group name(s):" \
                                && ret=0
                        ;;
                        *) _arguments \
                            "1: :_group_actions" \
                            "*::arg:->args" && ret=0
                        ;;
                    esac
                ;;
                version)
                    _message "no more arguments" && ret=0
                ;;
                *)
                    _message "unknown subcommand" && ret=1
                ;;
            esac
        ;;
    esac

    return ret
}

_pacdef


