#compdef pacdef

_pacdef() {
    integer ret=1
    local line
    function _actions {
        local -a actions
        actions=(
            'clean:uninstall packages not managed by pacdef'
            'edit:edit an imported group file'
            'groups:show names of imported groups'
            'import:import a new group file'
            'remove:remove a group file'
            'search:show the group containing a package'
            'show:show packages under an imported group'
            'sync:install all packages from imported groups'
            'unmanaged:show explicitly installed packages not managed by pacdef'
            'version:show version info'
        )
        _describe 'action' actions
    }

    _arguments \
        "1: :_actions" \
        "*::arg:->args"

    if [ -z "${XDG_CONFIG_HOME}" ]; then
        GROUPDIR="~/.config/pacdef/groups"
    else
        GROUPDIR="${XDG_CONFIG_HOME}/pacdef/groups"
    fi


    case $line[1] in
        edit)
            _arguments "1:group file:_files -W '$GROUPDIR'"
        ;;
        import)
            _arguments "*:new group file(s):_files"
        ;;
        search)
            _arguments "1:package:"
        ;;
        show)
            _arguments "1:group file:_files -W '$GROUPDIR'"
        ;;
        remove)
            _arguments "*:group file:_files -W '$GROUPDIR'"
        ;;
        "") ;;
        *)
            _message "no more arguments"
        ;;
    esac

    return ret
}

_pacdef
