_create__rust__app() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="create__rust__app"
                ;;
            create__rust__app,cache)
                cmd="create__rust__app__subcmd__cache"
                ;;
            create__rust__app,help)
                cmd="create__rust__app__subcmd__help"
                ;;
            create__rust__app__subcmd__help,cache)
                cmd="create__rust__app__subcmd__help__subcmd__cache"
                ;;
            create__rust__app__subcmd__help,help)
                cmd="create__rust__app__subcmd__help__subcmd__help"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        create__rust__app)
            opts="-i -v -t -a -f -h -V --info --verbose --template --addons --extend --set --config --force --no-install --skip-install --interactive --no-interactive --list-templates --list-addons --category --offline --no-cache --cache-dir --pin --refresh --strict-version --keep-on-failure --catalog-url --catalog-path --fixture --fixture-dir --add-completion --json --help --version cache help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --template)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --addons)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --extend)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --set)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --category)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cache-dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pin)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --refresh)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --catalog-url)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --catalog-path)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --fixture-dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --add-completion)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        create__subcmd__rust__subcmd__app__subcmd__cache)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        create__subcmd__rust__subcmd__app__subcmd__help)
            opts="cache help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        create__subcmd__rust__subcmd__app__subcmd__help__subcmd__cache)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        create__subcmd__rust__subcmd__app__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _create__rust__app -o nosort -o bashdefault -o default create-rust-app
else
    complete -F _create__rust__app -o bashdefault -o default create-rust-app
fi
