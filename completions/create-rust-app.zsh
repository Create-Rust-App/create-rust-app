#compdef create-rust-app

autoload -U is-at-least

_create-rust-app() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-t+[Template URL, file\://, or slug]:TEMPLATE:_default' \
'--template=[Template URL, file\://, or slug]:TEMPLATE:_default' \
'-a+[Comma-separated addon slugs or URLs]:ADDONS:_default' \
'--addons=[Comma-separated addon slugs or URLs]:ADDONS:_default' \
'--extend=[Alias for --addons (single value; prefer --addons)]:EXTEND:_default' \
'*--set=[Set key=value (repeatable)]:SETS:_default' \
'--config=[Use cra.config.json from a custom path (base for --set overlay)]:CONFIG:_default' \
'--category=[Filter --list-templates by category slug (matches entry tags)]:CATEGORY:_default' \
'--cache-dir=[Override CRA_CACHE_DIR]:CACHE_DIR:_default' \
'--pin=[Pin git ref]:PIN:_default' \
'--refresh=[Cache refresh policy\: always|stale|manual]:REFRESH:_default' \
'--catalog-url=[Override templates.json URL]:CATALOG_URL:_default' \
'--catalog-path=[Local templates.json path]:CATALOG_PATH:_default' \
'--fixture-dir=[Fixture catalog directory (implies --fixture)]:FIXTURE_DIR:_default' \
'--add-completion=[Print completion script\: bash|zsh|fish]:SHELL:_default' \
'-i[Print environment info]' \
'--info[Print environment info]' \
'-v[Verbose output]' \
'--verbose[Verbose output]' \
'-f[Allow non-empty target directory / skip clean confirm]' \
'--force[Allow non-empty target directory / skip clean confirm]' \
'--no-install[Skip the post-scaffold \`cargo check\`]' \
'--skip-install[Skip the post-scaffold \`cargo check\` (alias for --no-install)]' \
'--interactive[Interactive prompts]' \
'--no-interactive[Disable interactive prompts]' \
'--list-templates[List templates from catalog]' \
'--list-addons[List addons from catalog]' \
'--offline[Offline mode]' \
'--no-cache[Bypass catalog cache]' \
'--strict-version[Strict version checks]' \
'--keep-on-failure[Keep project dir on failure]' \
'--fixture[Use local fixtures/catalog templates.json]' \
'--json[JSON output for cache subcommands and catalog lists]' \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
'::project -- Project directory to create:_default' \
":: :_create-rust-app_commands" \
"*::: :->create-rust-app" \
&& ret=0
    case $state in
    (create-rust-app)
        words=($line[2] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:create-rust-app-command-$line[2]:"
        case $line[2] in
            (cache)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
'::action -- Cache action\: status | path | clean:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_create-rust-app__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:create-rust-app-help-command-$line[1]:"
        case $line[1] in
            (cache)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_create-rust-app_commands] )) ||
_create-rust-app_commands() {
    local commands; commands=(
'cache:Inspect or clean the local catalog cache' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'create-rust-app commands' commands "$@"
}
(( $+functions[_create-rust-app__subcmd__cache_commands] )) ||
_create-rust-app__subcmd__cache_commands() {
    local commands; commands=()
    _describe -t commands 'create-rust-app cache commands' commands "$@"
}
(( $+functions[_create-rust-app__subcmd__help_commands] )) ||
_create-rust-app__subcmd__help_commands() {
    local commands; commands=(
'cache:Inspect or clean the local catalog cache' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'create-rust-app help commands' commands "$@"
}
(( $+functions[_create-rust-app__subcmd__help__subcmd__cache_commands] )) ||
_create-rust-app__subcmd__help__subcmd__cache_commands() {
    local commands; commands=()
    _describe -t commands 'create-rust-app help cache commands' commands "$@"
}
(( $+functions[_create-rust-app__subcmd__help__subcmd__help_commands] )) ||
_create-rust-app__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'create-rust-app help help commands' commands "$@"
}

if [ "$funcstack[1]" = "_create-rust-app" ]; then
    _create-rust-app "$@"
else
    compdef _create-rust-app create-rust-app
fi
