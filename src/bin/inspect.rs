use anyhow::Result;
use gharry::{config::Config, env};

fn main() -> Result<()> {
    let setup = env::setup_exec_env()?;
    let config = Config::build_default(setup.work_dir)?;
    println!("{:?}", config);

    Ok(())
}
