/// Quickly check if the program is launched with ROOT permissions, and make it exit if it's not
/// the case.
pub fn root_check() {
    let current_uid = unsafe { libc::getuid() };
    if current_uid != 0 {
        eprintln!("Error: ROOT permissions are needed for this operation.");
        std::process::exit(1);
    }
}

pub fn enable_debug_logging_if_needed() {
    if cfg!(debug_assertions) && std::env::var_os("SQUIRTLE_IS_CHILD").is_none() {
        unsafe { std::env::set_var("RUST_LOG", "debug"); }
        env_logger::init();
    }
}