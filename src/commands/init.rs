use std::env;

use anyhow::Result;

use crate::cli::InitArgs;
use crate::shell::registry_from_str;

pub fn run(args: InitArgs) -> Result<()> {
    let shell = registry_from_str(&args.shell)?;
    let bin = env::current_exe()?;
    let script = shell.init_script(&bin)?;

    // Script → stdout so `eval "$(bro init bash)"` only captures the function
    print!("{}", script);

    // Hint → stderr
    // eprintln!("\n# To install, {}", install_hint(&args.shell)); 

    Ok(())
}

fn install_hint(shell: &str) -> &'static str {
    match shell.to_lowercase().as_str() {
        "bash"              => "add to ~/.bashrc:\n#   eval \"$(bro init bash)\"",
        "zsh"               => "add to ~/.zshrc:\n#   eval \"$(bro init zsh)\"",
        "fish"              => "add to ~/.config/fish/config.fish:\n#   bro init fish | source",
        "powershell" | "pwsh" =>
            "add to $PROFILE:\n#   Invoke-Expression (& bro init powershell | Out-String)",
        _                   => "see `bro init --help` for supported shells.",
    }
}
