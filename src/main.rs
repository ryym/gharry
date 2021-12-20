use anyhow::Result;
use gharry::{config::Config, env, polling};
use std::{thread, time::Duration};

fn main() -> Result<()> {
    let setup = env::setup_exec_env()?;
    let config = Config::build_default(setup.work_dir)?;
    'poll_loop: loop {
        match polling::run(&config) {
            Ok(()) => return Ok(()),
            Err(err) => {
                for cause in err.chain() {
                    if let Some(err) = cause.downcast_ref::<reqwest::Error>() {
                        log::info!("some web request failed: {}", err);
                        log::info!("will retry polling after {} seconds...", 30);
                        thread::sleep(Duration::from_secs(30));
                        continue 'poll_loop;
                    }
                }
                return Err(err);
            }
        }
    }
}
