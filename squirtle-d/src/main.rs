const DEFAULT_SOCKET_PATH: &str = "/run/squirtled.sock";

fn main() -> std::io::Result<()> {
    squirtle_core::utils::enable_debug_logging_if_needed();

    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        1 => squirtle_d::start(DEFAULT_SOCKET_PATH)?,
        2 => match args[1].as_str() {
            "--socket" => eprintln!("'--socket' argument passed without path"),
            _          => help()
        },
        3 => match args[1].as_str() {
            "--socket" => squirtle_d::start(args[2].as_str())?,
            _          => help()
        },
        _ => help()
    }
    Ok(())
}

fn help() {
    println!("SquirtleD available arguments:
--socket <path> : Tell where SquirtleD's UNIX socket should be created.
                  (By default, it is created at /run/squirtle.sock)");
}
