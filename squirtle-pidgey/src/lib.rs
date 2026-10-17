// NOTE to myself: Don't even try to finish this part before finalizing SquirtleD (including every IPC calls)
// Also maybe at some point Windwaker should go to Squirtle-Core, so Pidgey and other clients will be mainly there
// for configuration purposes.
// I'm really not sure doing ~20 IPC call per second for quering temperatures & adjusting fan speed is a good thing.
mod config;
mod windwaker;
use config::CONFIGURATION;

pub enum RunningMode {
    Client,
    Standalone,
}

pub fn init(mode: RunningMode) {
    match mode {
        crate::RunningMode::Client => windwaker::start(RunningMode::Client),
        crate::RunningMode::Standalone => windwaker::start(RunningMode::Standalone),
    }
}

pub fn version() -> String { String::from(env!("CARGO_PKG_VERSION")) }
