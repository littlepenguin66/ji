mod archive;
mod commands;
mod crypto;
mod error;
mod remote;
mod store;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ji",
    about = "ji(笈) — dotfiles management tool",
    long_about = "Package encrypted dotfiles into a single .ji file for safe cross-device migration.",
    version,
    long_version = concat!(
        env!("CARGO_PKG_VERSION"),
        "\nhttps://github.com/littlepenguin66/ji",
    ),
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize ji configuration and generate age keypair
    Init {
        /// Add an existing public key as recipient (can be specified multiple times)
        #[arg(long = "key", value_name = "PUBKEY")]
        keys: Vec<String>,

        /// Auto-generate age keypair (skip interactive prompts)
        #[arg(short = 'a')]
        auto: bool,

        /// Overwrite existing config.toml
        #[arg(long = "force")]
        force: bool,
    },

    /// Add files to the manifest
    Add {
        /// Paths to track (relative to $HOME)
        #[arg(required = true, value_name = "PATH")]
        paths: Vec<PathBuf>,

        /// Only include files matching this glob pattern
        #[arg(long = "include", value_name = "PATTERN")]
        include: Vec<String>,

        /// Exclude files matching this glob pattern
        #[arg(long = "exclude", value_name = "PATTERN")]
        exclude: Vec<String>,
    },

    /// Remove files from the manifest
    Rm {
        /// Paths to untrack
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,

        /// Remove all tracked files
        #[arg(long = "all")]
        all: bool,
    },

    /// List tracked files and their checksums
    List {
        /// Output in JSON format
        #[arg(long = "json")]
        json: bool,

        /// Show file sizes and checksums
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },

    /// Show file change status
    Status {
        /// Compact output (only path and status marker)
        #[arg(short = 's', long = "short")]
        short: bool,
    },

    /// Pack tracked files into an encrypted .ji archive
    Pack {
        /// Output path (default: ~/.local/share/ji/<hostname>.ji)
        #[arg(short = 'o', value_name = "PATH")]
        output: Option<PathBuf>,

        /// Refuse to pack if any checksum mismatches
        #[arg(long = "strict")]
        strict: bool,

        /// Show diff of changed files during pack
        #[arg(long = "verbose")]
        verbose: bool,
    },

    /// Unpack a .ji archive and restore files to $HOME
    Unpack {
        /// The .ji file to unpack
        #[arg(value_name = "INPUT.ji")]
        input: PathBuf,

        /// Show what would be done without actually doing it
        #[arg(long = "dry-run")]
        dry_run: bool,

        /// Overwrite existing files without prompting
        #[arg(long = "force")]
        force: bool,

        /// Ask before overwriting each file
        #[arg(long = "interactive")]
        interactive: bool,

        /// Backup existing files as .bak before overwriting
        #[arg(long = "backup")]
        backup: bool,
    },

    /// Verify .ji file integrity
    Check {
        #[arg(value_name = "INPUT.ji")]
        input: Option<PathBuf>,

        #[arg(long = "deep")]
        deep: bool,
    },

    /// Show unified diff of changed files
    Diff {
        /// Limit diff to a specific file (default: all changed files)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,
    },

    /// Bidirectional sync with a remote
    Sync {
        /// Remote name to sync with
        #[arg(value_name = "REMOTE")]
        remote: String,
    },

    /// Manage remote endpoints
    #[command(subcommand)]
    Remote(RemoteCommand),

    /// Push a .ji file to a remote endpoint
    Push {
        /// Remote name
        #[arg(value_name = "NAME")]
        name: String,

        /// The .ji file to push
        #[arg(value_name = "INPUT.ji")]
        input: PathBuf,
    },

    /// Pull a .ji file from a remote endpoint
    Pull {
        /// Remote name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Manage recipients of a .ji file
    #[command(subcommand)]
    Recipient(RecipientCommand),

    /// Diagnose configuration, keys, and connectivity
    Doctor {
        /// Full check including remote connectivity and archive scan
        #[arg(long = "full")]
        full: bool,

        /// Output in JSON format
        #[arg(long = "json")]
        json: bool,
    },

    /// Generate shell completion script
    Completion {
        /// Target shell (bash, zsh, fish)
        #[arg(value_name = "SHELL")]
        shell: String,
    },
}

#[derive(Subcommand)]
enum RemoteCommand {
    /// Add a new remote endpoint
    Add {
        /// Remote name
        #[arg(value_name = "NAME")]
        name: String,

        /// Remote type (webdav, ssh)
        #[arg(long = "type", value_name = "TYPE")]
        remote_type: String,

        /// Remote URL
        #[arg(long = "url", value_name = "URL")]
        url: String,

        /// Authentication username
        #[arg(long = "user", value_name = "USER")]
        user: Option<String>,
    },

    /// Remove a remote endpoint
    Remove {
        /// Remote name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// List configured remote endpoints
    List {
        /// Output in JSON format
        #[arg(long = "json")]
        json: bool,
    },

    /// Test connectivity to a remote endpoint
    Test {
        /// Remote name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// List files on a remote endpoint
    Files {
        /// Remote name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Delete a file from a remote endpoint
    Delete {
        /// Remote name
        #[arg(value_name = "NAME")]
        name: String,

        /// File to delete
        #[arg(value_name = "FILE")]
        file: String,
    },
}

#[derive(Subcommand)]
enum RecipientCommand {
    /// List recipients of a .ji file
    List {
        #[arg(value_name = "INPUT.ji")]
        input: Option<PathBuf>,
    },

    /// Add a recipient to a .ji file
    Add {
        /// Public key to add
        #[arg(long = "key", value_name = "PUBKEY")]
        key: String,

        /// The .ji file to modify
        #[arg(value_name = "INPUT.ji")]
        input: PathBuf,
    },

    /// Remove a recipient from a .ji file
    Remove {
        /// Public key to remove
        #[arg(long = "key", value_name = "PUBKEY")]
        key: String,

        /// The .ji file to modify
        #[arg(value_name = "INPUT.ji")]
        input: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match run(cli) {
        Ok(()) => {}
        Err(error::Error::HasChanges) => {
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("ji: {e}");
            std::process::exit(1);
        }
    }
}

fn run(cli: Cli) -> error::Result<()> {
    use Command::*;

    match cli.command {
        Init { keys, auto, force } => commands::init::run(keys, auto, force),
        Add {
            paths,
            include,
            exclude,
        } => commands::add::run(paths, include, exclude),
        Rm { paths, all } => commands::rm::run(paths, all),
        List { json, verbose } => commands::list::run(json, verbose),
        Status { short } => commands::status::run(short),
        Pack {
            output,
            strict,
            verbose,
        } => commands::pack::run(output, strict, verbose),
        Unpack {
            input,
            dry_run,
            force,
            interactive,
            backup,
        } => commands::unpack::run(input, dry_run, force, interactive, backup),
        Check { input, deep } => commands::check::run(input, deep),
        Diff { path } => commands::diff::run(path),
        Sync { remote } => commands::sync::run(remote),
        Remote(rc) => match rc {
            RemoteCommand::Add {
                name,
                remote_type,
                url,
                user,
            } => commands::remote::run_add(name, remote_type, url, user),
            RemoteCommand::Remove { name } => commands::remote::run_remove(name),
            RemoteCommand::List { json } => commands::remote::run_list(json),
            RemoteCommand::Test { name } => commands::remote::run_test(name),
            RemoteCommand::Files { name } => commands::remote::run_files(name),
            RemoteCommand::Delete { name, file } => commands::remote::run_delete(name, &file),
        },
        Push { name, input } => commands::push::run(name, input),
        Pull { name } => commands::pull::run(name),
        Recipient(rc) => match rc {
            RecipientCommand::List { input } => commands::recipient::run_list(input),
            RecipientCommand::Add { key, input } => commands::recipient::run_add(key, input),
            RecipientCommand::Remove { key, input } => commands::recipient::run_remove(key, input),
        },
        Doctor { full, json } => commands::doctor::run(full, json),
        Completion { shell } => commands::completion::run(shell),
    }
}
