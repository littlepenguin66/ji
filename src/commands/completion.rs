use crate::error::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

pub fn run(shell: String) -> Result<()> {
    let shell = match shell.as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        _ => {
            return Err(crate::error::Error::Other(format!(
                "unsupported shell: {shell}. Supported: bash, zsh, fish"
            )));
        }
    };

    let mut cmd = crate::Cli::command();
    let mut buf = Vec::new();
    generate(shell, &mut cmd, "ji", &mut buf);

    let output = String::from_utf8_lossy(&buf);
    if shell == Shell::Fish {
        for line in output.lines() {
            if line.trim() == "string join \\n h/help V/version" {
                println!("string join '\\n' h/help V/version");
            } else {
                println!("{line}");
            }
        }
    } else {
        print!("{output}");
    }

    print_dynamic(shell);

    Ok(())
}

fn print_dynamic(shell: Shell) {
    match shell {
        Shell::Fish => {
            println!();
            println!("function __ji_list_files");
            println!("    ji list --json 2>/dev/null | jq -r 'keys[]' 2>/dev/null");
            println!("end");
            println!();
            println!("function __ji_list_remotes");
            println!("    ji remote list --json 2>/dev/null | jq -r '.[].name' 2>/dev/null");
            println!("end");
            println!();
            println!("complete -c ji -n \"__fish_seen_subcommand_from rm\" -a \"(__ji_list_files)\"");
            println!("complete -c ji -n \"__fish_seen_subcommand_from push; and test (count (commandline -opc)) -eq 2\" -a \"(__ji_list_remotes)\"");
            println!("complete -c ji -n \"__fish_seen_subcommand_from push; and test (count (commandline -opc)) -gt 2\" -a \"(ls *.ji 2>/dev/null)\"");
            println!("complete -c ji -n \"__fish_seen_subcommand_from pull\" -a \"(__ji_list_remotes)\"");
            println!("complete -c ji -n \"__fish_seen_subcommand_from sync\" -a \"(__ji_list_remotes)\"");
            println!("complete -c ji -n \"__fish_seen_subcommand_from remote; and __fish_seen_subcommand_from remove; or __fish_seen_subcommand_from test; or __fish_seen_subcommand_from files; or __fish_seen_subcommand_from delete\" -a \"(__ji_list_remotes)\"");
            println!("complete -c ji -n \"__fish_seen_subcommand_from unpack; or __fish_seen_subcommand_from check\" -a \"(ls *.ji 2>/dev/null)\"");
            println!("complete -c ji -n \"__fish_seen_subcommand_from recipient; and __fish_seen_subcommand_from list; or __fish_seen_subcommand_from add; or __fish_seen_subcommand_from remove\" -a \"(ls *.ji 2>/dev/null)\"");
        }
        _ => {}
    }
}
