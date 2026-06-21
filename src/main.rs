mod cli;
mod classify;
mod commands;
mod config;
mod exec;
mod resolve;
mod shell;
mod store;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Cmd};

fn main() -> Result<()> {
    // Fast path: dynamic alias-name completion for shell scripts.
    // Called as `bro --complete <prefix>` by the generated completion scripts.
    let raw: Vec<String> = std::env::args().collect();
    if let Some(i) = raw.iter().position(|a| a == "--complete") {
        let prefix = raw.get(i + 1).map(String::as_str).unwrap_or("");
        return commands::completions::print_completions(prefix);
    }

    let cli = Cli::parse();

    let ctx = exec::Context {
        emit: cli.emit,
        shell_name: cli.shell_name.as_deref().unwrap_or("bash").to_string(),
        exec_file: cli.exec_file,
        dry_run: cli.dry_run,
    };

    match cli.cmd {
        Cmd::Add(args)         => commands::add::run(args),
        Cmd::Update(args)      => commands::update::run(args),
        Cmd::Remove(args)      => commands::remove::run(args),
        Cmd::List(args)        => commands::list::run(args),
        Cmd::Info(args)        => commands::info::run(args),
        Cmd::Search(args)      => commands::search::run(args),
        Cmd::Init(args)        => commands::init::run(args),
        Cmd::Paths             => commands::paths::run(),
        Cmd::Edit(args)        => commands::edit::run(args),
        Cmd::Completions(args) => commands::completions::run(args),
        Cmd::Run(args)         => exec::run_cmd(args, &ctx),
        Cmd::External(v)       => exec::run_external(v, &ctx),
    }
}
