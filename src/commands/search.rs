use anyhow::Result;

use crate::cli::SearchArgs;
use crate::config::{global_store_path, project_store_path};
use crate::store::Store;

pub fn run(args: SearchArgs) -> Result<()> {
    let kw = args.keyword.to_lowercase();

    let project_path  = project_store_path();
    let project_store = project_path.as_deref().map(Store::load).transpose()?.unwrap_or_default();
    let global_store  = Store::load(&global_store_path())?;

    let mut hits: Vec<(String, &'static str, String)> = Vec::new();

    let matches_kw = |name: &str, alias: &crate::store::model::Alias| {
        name.to_lowercase().contains(&kw)
            || alias.cmd.to_lowercase().contains(&kw)
            || alias.desc.as_deref().unwrap_or("").to_lowercase().contains(&kw)
            || alias.tags.iter().any(|t| t.to_lowercase().contains(&kw))
    };

    for (name, alias) in &project_store.aliases {
        if matches_kw(name, alias) {
            hits.push((name.clone(), "project", alias.cmd.clone()));
        }
    }
    for (name, alias) in &global_store.aliases {
        if matches_kw(name, alias) {
            let src = if project_store.get(name).is_some() { "global (shadowed)" } else { "global" };
            hits.push((name.clone(), src, alias.cmd.clone()));
        }
    }

    if hits.is_empty() {
        eprintln!("no matches for '{}'", args.keyword);
        return Ok(());
    }

    hits.sort_by(|a, b| a.0.cmp(&b.0));
    let w = hits.iter().map(|h| h.0.len()).max().unwrap_or(4);
    for (name, src, cmd) in &hits {
        eprintln!("{:<w$}  [{src}]  {cmd}", name, w = w);
    }

    Ok(())
}
