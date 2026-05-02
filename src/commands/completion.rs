use crate::error::Result;

pub fn run(shell: String) -> Result<()> {
    match shell.as_str() {
        "bash" => print_bash(),
        "zsh" => print_zsh(),
        "fish" => print_fish(),
        _ => {
            return Err(crate::error::Error::Other(format!(
                "unsupported shell: {shell}. Supported: bash, zsh, fish"
            )));
        }
    }
    Ok(())
}

fn subcommands() -> &'static str {
    "init add rm list status pack unpack check diff sync remote push pull recipient doctor completion"
}

fn print_bash() {
    print!(
        r#"_ji() {{
    local cur prev words cword
    _init_completion || return

    case "${{words[1]}}" in
        rm)
            COMPREPLY=($(compgen -W "$(ji list --json 2>/dev/null | jq -r '.files | keys[]' 2>/dev/null)" -- "$cur"))
            ;;
        remote)
            case "${{words[2]}}" in
                remove|test|files|delete)
                    COMPREPLY=($(compgen -W "$(ji remote list --json 2>/dev/null | jq -r '.[].name' 2>/dev/null)" -- "$cur"))
                    ;;
            esac
            ;;
        push)
            case "${{#words[@]}}" in
                2) COMPREPLY=($(compgen -W "$(ji remote list --json 2>/dev/null | jq -r '.[].name' 2>/dev/null)" -- "$cur")) ;;
                3) COMPREPLY=($(compgen -f -X '!*.ji' -- "$cur")) ;;
            esac
            ;;
        pull|sync)
            COMPREPLY=($(compgen -W "$(ji remote list --json 2>/dev/null | jq -r '.[].name' 2>/dev/null)" -- "$cur"))
            ;;
        unpack|check)
            COMPREPLY=($(compgen -f -X '!*.ji' -- "$cur"))
            ;;
        recipient)
            COMPREPLY=($(compgen -f -X '!*.ji' -- "$cur"))
            ;;
        *)
            COMPREPLY=($(compgen -W "{}" -- "$cur"))
            ;;
    esac
}}

complete -F _ji ji
"#, subcommands()
    );
}

fn print_zsh() {
    print!(
        r#"#compdef ji

_ji_commands() {{
    local -a commands
    commands=(
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
        'doctor:Diagnose config keys and connectivity'
        'completion:Generate shell completion'
    )
    _describe 'command' commands
}}

_ji_remote_subcommands() {{
    local -a subs
    subs=(
        'add:Add a remote endpoint'
        'remove:Remove a remote endpoint'
        'list:List configured remotes'
        'test:Test connectivity'
        'files:List files on remote'
        'delete:Delete file from remote'
    )
    _describe 'subcommand' subs
}}

_ji_recipient_subcommands() {{
    local -a subs
    subs=(
        'list:List recipients'
        'add:Add a recipient'
        'remove:Remove a recipient'
    )
    _describe 'subcommand' subs
}}

_ji() {{
    local state line
    typeset -A opt_args

    _arguments '1: :->command' '*: :->args'

    case $state in
        command)
            _ji_commands
            ;;
        args)
            case $words[1] in
                remote)
                    _ji_remote_subcommands
                    ;;
                recipient)
                    _ji_recipient_subcommands
                    ;;
                rm)
                    local -a files
                    files=(${{(f)"$(ji list --json 2>/dev/null | jq -r '.files | keys[]' 2>/dev/null)"}})
                    _values 'files' $files
                    ;;
                push)
                    if [[ $CURRENT -eq 2 ]]; then
                        local -a remotes
                        remotes=(${{(f)"$(ji remote list --json 2>/dev/null | jq -r '.[].name' 2>/dev/null)"}})
                        _values 'remote' $remotes
                    elif [[ $CURRENT -eq 3 ]]; then
                        _files -g '*.ji'
                    fi
                    ;;
                pull|sync)
                    local -a remotes
                    remotes=(${{(f)"$(ji remote list --json 2>/dev/null | jq -r '.[].name' 2>/dev/null)"}})
                    _values 'remote' $remotes
                    ;;
                unpack|check)
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

fn print_fish() {
    print!(
        r#"function __ji_list_files
    ji list --json 2>/dev/null | jq -r '.files | keys[]' 2>/dev/null
end

function __ji_list_remotes
    ji remote list --json 2>/dev/null | jq -r '.[].name' 2>/dev/null
end

complete -c ji -f

{}
complete -c ji -n "not __fish_seen_subcommand_from {}" -a completion -d "Shell completion"

complete -c ji -n "__fish_seen_subcommand_from rm" -a "(__ji_list_files)"
complete -c ji -n "__fish_seen_subcommand_from push" -a "(__ji_list_remotes)"
complete -c ji -n "__fish_seen_subcommand_from push" -F -g '*.ji'
complete -c ji -n "__fish_seen_subcommand_from pull" -a "(__ji_list_remotes)"
complete -c ji -n "__fish_seen_subcommand_from sync" -a "(__ji_list_remotes)"
complete -c ji -n "__fish_seen_subcommand_from remote; and __fish_seen_subcommand_from remove; or __fish_seen_subcommand_from test; or __fish_seen_subcommand_from files; or __fish_seen_subcommand_from delete" -a "(__ji_list_remotes)"
complete -c ji -n "__fish_seen_subcommand_from unpack; or __fish_seen_subcommand_from check" -F -g '*.ji'
complete -c ji -n "__fish_seen_subcommand_from recipient; and __fish_seen_subcommand_from list; or __fish_seen_subcommand_from add; or __fish_seen_subcommand_from remove" -F -g '*.ji'
"#,
        fish_command_lines(),
        subcommands(),
    );
}

fn fish_command_lines() -> String {
    let cmds = vec![
        ("init", "Initialize"),
        ("add", "Add files"),
        ("rm", "Remove files"),
        ("list", "List files"),
        ("status", "File status"),
        ("pack", "Pack archive"),
        ("unpack", "Unpack archive"),
        ("check", "Verify archive"),
        ("diff", "Show diffs"),
        ("sync", "Sync with remote"),
        ("remote", "Manage remotes"),
        ("push", "Push to remote"),
        ("pull", "Pull from remote"),
        ("recipient", "Manage recipients"),
        ("doctor", "Diagnose"),
    ];
    cmds.iter()
        .map(|(name, desc)| {
            format!(
                "complete -c ji -n \"not __fish_seen_subcommand_from {}\" -a {name} -d \"{desc}\"",
                subcommands()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
