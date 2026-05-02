use crate::error::Result;

pub fn run(shell: String) -> Result<()> {
    match shell.as_str() {
        "bash" => {
            print!(
                r#"_ji() {{
    local cur prev words cword
    _init_completion || return

    case "${{words[1]}}" in
        rm)
            COMPREPLY=($(compgen -W "$(ji list --json 2>/dev/null | grep -o '"\.\?[^"]*"' | tr -d '"')" -- "$cur"))
            ;;
        remote)
            case "${{words[2]}}" in
                remove|test)
                    COMPREPLY=($(compgen -W "$(ji remote list --json 2>/dev/null | grep -o '"name":"[^"]*"' | cut -d'"' -f4)" -- "$cur"))
                    ;;
            esac
            ;;
        push)
            if [[ ${{#words[@]}} -eq 3 ]]; then
                COMPREPLY=($(compgen -W "$(ji remote list --json 2>/dev/null | grep -o '"name":"[^"]*"' | cut -d'"' -f4)" -- "$cur"))
            fi
            ;;
        pull|sync)
            COMPREPLY=($(compgen -W "$(ji remote list --json 2>/dev/null | grep -o '"name":"[^"]*"' | cut -d'"' -f4)" -- "$cur"))
            ;;
        unpack|check|recipient)
            COMPREPLY=($(compgen -f -X '!*.ji' -- "$cur"))
            ;;
    esac
}}

complete -F _ji ji
"#
            );
        }
        "zsh" => {
            print!(
                r#"#compdef ji

local -a _ji_commands
_ji_commands=(
    'init:Initialize ji config and generate age keypair'
    'add:Add files to the manifest'
    'rm:Remove files from the manifest'
    'list:List tracked files'
    'status:Show file change status'
    'pack:Pack tracked files into .ji archive'
    'unpack:Unpack a .ji archive'
    'check:Verify .ji integrity'
    'diff:Show diff of changed files'
    'sync:Bidirectional sync with remote'
    'remote:Manage remote endpoints'
    'push:Push .ji to remote'
    'pull:Pull .ji from remote'
    'recipient:Manage .ji recipients'
    'completion:Generate shell completion'
)

_ji() {{
    local state
    _arguments '1: :->command' '*:: :->args'

    case $state in
        command)
            _describe 'command' _ji_commands
            ;;
        args)
            case $words[1] in
                rm)
                    local -a files
                    files=(${{(f)"$(ji list --json 2>/dev/null | jq -r 'keys[]' 2>/dev/null)"}})
                    _values 'files' $files
                    ;;
                push)
                    if [[ $CURRENT -eq 2 ]]; then
                        local -a remotes
                        remotes=(${{(f)"$(ji remote list --json 2>/dev/null | jq -r '.[].name' 2>/dev/null)"}})
                        _values 'remote' $remotes
                    fi
                    ;;
                pull|sync)
                    local -a remotes
                    remotes=(${{(f)"$(ji remote list --json 2>/dev/null | jq -r '.[].name' 2>/dev/null)"}})
                    _values 'remote' $remotes
                    ;;
                unpack|check|recipient)
                    _files -g '*.ji'
                    ;;
            esac
            ;;
    esac
}}

_ji
"#
            );
        }
        "fish" => {
            print!(
                r#"function __ji_list_files
    ji list --json 2>/dev/null | jq -r 'keys[]' 2>/dev/null
end

function __ji_list_remotes
    ji remote list --json 2>/dev/null | jq -r '.[].name' 2>/dev/null
end

complete -c ji -f

complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a init -d "Initialize"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a add -d "Add files"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a rm -d "Remove files"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a list -d "List files"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a status -d "File status"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a pack -d "Pack archive"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a unpack -d "Unpack archive"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a check -d "Verify archive"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a diff -d "Show diffs"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a sync -d "Sync"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a remote -d "Manage remotes"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a push -d "Push"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a pull -d "Pull"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a recipient -d "Manage recipients"
complete -c ji -n "not __fish_seen_subcommand_from init add rm list status pack unpack check diff sync remote push pull recipient completion" -a completion -d "Shell completion"

# Dynamic completions for rm
complete -c ji -n "__fish_seen_subcommand_from rm" -a "(__ji_list_files)"

# Dynamic completions for push/pull/sync
complete -c ji -n "__fish_seen_subcommand_from push" -a "(__ji_list_remotes)" -d "Remote"
complete -c ji -n "__fish_seen_subcommand_from pull" -a "(__ji_list_remotes)" -d "Remote"
complete -c ji -n "__fish_seen_subcommand_from sync" -a "(__ji_list_remotes)" -d "Remote"

# .ji files for unpack/check/recipient
complete -c ji -n "__fish_seen_subcommand_from unpack; or __fish_seen_subcommand_from check; or __fish_seen_subcommand_from recipient" -F -g '*.ji'
"#
            );
        }
        _ => {
            return Err(crate::error::Error::Other(format!(
                "unsupported shell: {shell}. Supported: bash, zsh, fish"
            )));
        }
    }

    Ok(())
}
