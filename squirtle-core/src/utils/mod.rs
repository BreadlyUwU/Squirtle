pub fn rootcheck() {
    let current_uid = unsafe { libc::getuid() };
    if current_uid != 0 {
        eprintln!("Error: ROOT permissions are needed for this operation.");
        std::process::exit(1);
    }
}
