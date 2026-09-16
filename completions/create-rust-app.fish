# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_create_rust_app_global_optspecs
    string join \n i/info v/verbose t/template= a/addons= extend= set= config= f/force no-install skip-install interactive no-interactive list-templates list-addons category= offline no-cache cache-dir= pin= refresh= strict-version keep-on-failure catalog-url= catalog-path= fixture fixture-dir= add-completion= json h/help V/version
end

function __fish_create_rust_app_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_create_rust_app_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_create_rust_app_using_subcommand
    set -l cmd (__fish_create_rust_app_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -s t -l template -d 'Template URL, file://, or slug' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -s a -l addons -d 'Comma-separated addon slugs or URLs' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l extend -d 'Alias for --addons (single value; prefer --addons)' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l set -d 'Set key=value (repeatable)' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l config -d 'Use cra.config.json from a custom path (base for --set overlay)' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l category -d 'Filter --list-templates by category slug (matches entry tags)' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l cache-dir -d 'Override CRA_CACHE_DIR' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l pin -d 'Pin git ref' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l refresh -d 'Cache refresh policy: always|stale|manual' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l catalog-url -d 'Override templates.json URL' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l catalog-path -d 'Local templates.json path' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l fixture-dir -d 'Fixture catalog directory (implies --fixture)' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l add-completion -d 'Print completion script: bash|zsh|fish' -r
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -s i -l info -d 'Print environment info'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -s v -l verbose -d 'Verbose output'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -s f -l force -d 'Allow non-empty target directory / skip clean confirm'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l no-install -d 'Skip the post-scaffold `cargo check`'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l skip-install -d 'Skip the post-scaffold `cargo check` (alias for --no-install)'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l interactive -d 'Interactive prompts'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l no-interactive -d 'Disable interactive prompts'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l list-templates -d 'List templates from catalog'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l list-addons -d 'List addons from catalog'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l offline -d 'Offline mode'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l no-cache -d 'Bypass catalog cache'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l strict-version -d 'Strict version checks'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l keep-on-failure -d 'Keep project dir on failure'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l fixture -d 'Use local fixtures/catalog templates.json'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -l json -d 'JSON output for cache subcommands and catalog lists'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -s h -l help -d 'Print help'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -s V -l version -d 'Print version'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -a "cache" -d 'Inspect or clean the local catalog cache'
complete -c create-rust-app -n "__fish_create_rust_app_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c create-rust-app -n "__fish_create_rust_app_using_subcommand cache" -s h -l help -d 'Print help'
complete -c create-rust-app -n "__fish_create_rust_app_using_subcommand help; and not __fish_seen_subcommand_from cache help" -f -a "cache" -d 'Inspect or clean the local catalog cache'
complete -c create-rust-app -n "__fish_create_rust_app_using_subcommand help; and not __fish_seen_subcommand_from cache help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
