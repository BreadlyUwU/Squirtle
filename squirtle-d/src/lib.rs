mod socket;

pub fn start() { socket::start().unwrap() }

pub fn version() -> String { String::from(env!("CARGO_PKG_VERSION")) }