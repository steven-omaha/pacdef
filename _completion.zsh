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
            'group:manage groups'
            'g:manage groups'
            'package:manage packages'
            'p:manage packages'
            'version:show version'
        )
        _describe 'pacdef subcommand' subcommands
    }

    function _group_actions {
        local -a group_actions
        group_actions=(
            'e:edit an imported group file'
            'edit:edit an imported group file'
            'l:show names of imported groups'
            'list:show names of imported groups'
            'i:import a new group file'
            'import:import a new group file'
            'n:create a new group file'
            'new:create a new group file'
            'r:remove a group file'
            'remove:remove a group file'
            's:show packages under an imported group'
            'show:show packages under an imported group'
        )
        _describe 'pacdef group action' group_actions
    }


    function _package_actions {
        local -a package_actions
        package_actions=(
            'c:uninstall packages not managed by pacdef'
            'clean:uninstall packages not managed by pacdef'
            'r:review unmanaged packages'
            'review:review unmanaged packages'
            'se:show the group containing a package'
            'search:show the group containing a package'
            'sy:install all packages from imported groups'
            'sync:install all packages from imported groups'
            'u:show explicitly installed packages not managed by pacdef'
            'unmanaged:show explicitly installed packages not managed by pacdef'
        )
        _describe 'pacdef package action' package_actions
    }

    _arguments -C \
        "1: :_subcommands" \
        "*::arg:->args" \
        && ret=0

    case $state in
        (args)
            case $line[1] in
                (p|package)
                    case $line[2] in
                        (se|search)
                            _arguments \
                                "2:regex:" && ret=0
                        ;;
                        (c|clean|r|review|sy|sync|u|unmanaged)
                            _message "no more arguments" && ret=0
                        ;;
                        *)
                            _arguments \
                                "1: :_package_actions" \
                                "*::arg:->args" && ret=0
                        ;;
                    esac
                ;;
                (g|group)
                    case $line[2] in
                        (l|list)
                            _message "no more arguments" && ret=0
                        ;;
                        (e|edit|r|remove|s|show)
                            _arguments "*:group file(s):_files -W '$GROUPDIR'" && ret=0

                        ;;
                        (i|import)
                            _arguments "*:new group file(s):_files" && ret=0
                        ;;
                        (n|new)
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


