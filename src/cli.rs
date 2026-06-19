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
