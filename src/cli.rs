use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "bro", about = "Personal command alias manager")]
pub struct Cli {
    /// Emit shell code to stdout instead of executing (set by wrapper).
    #[arg(long, global = true, hide = true)]
    pub emit: bool,

    /// Shell variant for quoting and wrapper generation.
    #[arg(long = "shell-name", global = true, hide = true, value_name = "SHELL")]
    pub shell_name: Option<String>,

    /// Temp file path for cmd.exe TempFileCall injection (set by bro.bat wrapper).
    #[arg(long, global = true, hide = true, value_name = "PATH")]
    pub exec_file: Option<String>,

    /// Print the resolved command without running or emitting it.
    #[arg(long, short = 'n', global = true)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Add an alias.
    Add(AddArgs),

    /// Update an existing alias.
    #[command(visible_alias = "set")]
    Update(UpdateArgs),

    /// Remove an alias.
    #[command(visible_alias = "rm")]
    Remove(RemoveArgs),

    /// List all aliases.
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// Show info about an alias.
    Info(InfoArgs),

    /// Search alias names and commands.
    #[command(visible_alias = "find")]
    Search(SearchArgs),

    /// Emit shell wrapper function (run once to install).
    Init(InitArgs),

    /// Show config file paths (global store, project store).
    Paths,

    /// Open the alias store (or a specific alias) in $EDITOR.
    Edit(EditArgs),

    /// Emit shell tab-completion script.
    Completions(CompletionsArgs),

    /// Run one or more aliases explicitly.
    Run(RunArgs),

    /// `bro <alias> [args...]` — resolved here.
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(clap::Args, Debug)]
pub struct AddArgs {
    /// Alias name.
    pub name: String,

    /// Command or path (required unless --py / --js).
    pub value: Option<String>,

    /// Register a Python script: expands to `python <path>`.
    #[arg(long, value_name = "PATH", conflicts_with = "js")]
    pub py: Option<String>,

    /// Register a JS script: expands to `node <path>`.
    #[arg(long, value_name = "PATH")]
    pub js: Option<String>,

    /// Force shell = true (persists cd, source, etc.).
    #[arg(long, conflicts_with = "no_shell")]
    pub shell: bool,

    /// Force shell = false.
    #[arg(long)]
    pub no_shell: bool,

    /// Short description.
    #[arg(long, short)]
    pub desc: Option<String>,

    /// Add one or more tags (repeatable: --tag k8s --tag infra).
    #[arg(long, value_name = "TAG")]
    pub tag: Vec<String>,

    /// Prompt for confirmation before running.
    #[arg(long, conflicts_with = "no_confirm")]
    pub confirm: bool,

    /// Never prompt for confirmation.
    #[arg(long)]
    pub no_confirm: bool,

    /// Write to project .bro instead of global store.
    #[arg(long, short)]
    pub local: bool,
}

#[derive(clap::Args, Debug)]
pub struct UpdateArgs {
    /// Alias name.
    pub name: String,

    /// New command value.
    pub value: Option<String>,

    #[arg(long, value_name = "PATH", conflicts_with = "js")]
    pub py: Option<String>,

    #[arg(long, value_name = "PATH")]
    pub js: Option<String>,

    #[arg(long, conflicts_with = "no_shell")]
    pub shell: bool,

    #[arg(long)]
    pub no_shell: bool,

    #[arg(long, short)]
    pub desc: Option<String>,

    /// Replace tags (repeatable: --tag k8s --tag infra).
    #[arg(long, value_name = "TAG")]
    pub tag: Vec<String>,

    #[arg(long, conflicts_with = "no_confirm")]
    pub confirm: bool,

    #[arg(long)]
    pub no_confirm: bool,

    #[arg(long, short)]
    pub local: bool,
}

#[derive(clap::Args, Debug)]
pub struct RemoveArgs {
    /// Alias name.
    pub name: String,

    /// Remove from project .bro instead of global store.
    #[arg(long, short)]
    pub local: bool,
}

#[derive(clap::Args, Debug)]
pub struct ListArgs {
    /// Show only project aliases.
    #[arg(long)]
    pub local: bool,

    /// Show only global aliases.
    #[arg(long, conflicts_with = "local")]
    pub global: bool,

    /// Filter by tag.
    #[arg(long, value_name = "TAG")]
    pub tag: Option<String>,

    /// Sort by run count (most used first).
    #[arg(long)]
    pub by_usage: bool,
}

#[derive(clap::Args, Debug)]
pub struct InfoArgs {
    /// Alias name.
    pub name: String,
}

#[derive(clap::Args, Debug)]
pub struct SearchArgs {
    /// Keyword to search in names and command values.
    pub keyword: String,
}

#[derive(clap::Args, Debug)]
pub struct InitArgs {
    /// Shell to emit wrapper for (bash, zsh, fish, powershell, cmd).
    pub shell: String,
}

#[derive(clap::Args, Debug)]
pub struct EditArgs {
    /// Alias name to jump to (opens the store containing it).
    pub name: Option<String>,

    /// Edit project .bro instead of global store.
    #[arg(long, short)]
    pub local: bool,
}

#[derive(clap::Args, Debug)]
pub struct CompletionsArgs {
    /// Shell to generate completion script for (bash, zsh, fish, powershell).
    pub shell: String,
}

#[derive(clap::Args, Debug)]
pub struct RunArgs {
    /// Comma-separated alias chain (e.g. a,b,c).
    #[arg(short = 'c', long, value_name = "CHAIN")]
    pub chain: Option<String>,

    /// Single alias name (positional, used when not chaining).
    pub name: Option<String>,

    /// Extra args passed to the alias command.
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}
