pub fn rootcheck() {
    let current_uid = unsafe { libc::getuid() };
    if current_uid != 0 {
        eprintln!("Error: Pidgey in standalone mode must be launched with ROOT permissions.");
        std::process::exit(1);
    }
}