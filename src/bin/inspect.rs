use anyhow::Result;
use gharry::{config::Config, env, inspect};

fn main() -> Result<()> {
    let setup = env::setup_exec_env()?;
    let config = Config::build_default(setup.work_dir)?;
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let limit = args.get(1).map(|v| v.as_str());
    let send_notifs = args.get(2).map(|v| v == "notify").unwrap_or(false);
    inspect::run(&config, &args[0], limit, send_notifs)?;

    Ok(())
}
