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

    Ok(())
}
