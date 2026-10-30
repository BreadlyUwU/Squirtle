mod daemon_ctl;
use squirtle_core::utils;

fn main() -> std::io::Result<()> {
    if std::env::consts::OS != "linux" {
        eprintln!("This program is designed to be run on Linux.\nOther UNIX-based OS aren't guaranteed to be supported in the future.\nSorry for that.\n");
        std::process::exit(1)
    }
    utils::enable_debug_logging_if_needed();

    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        2 => match args[1].as_str() { _ => help(), },
        3 => match args[1].as_str() {
            "daemon"      => match args[2].as_str() {
                "start"   => daemon_ctl::start(),
                "stop"    => daemon_ctl::stop(),
                _         => help(),
            },
            "fan-control" => match args[2].as_str() {
                "start"   => todo!(),
                _         => help()
            },
            _             => help()
        },
        4 => match args[1].as_str() {
            "fan-control"          => match args[2].as_str() {
                "start"            => match args[3].as_str() {
                    "--standalone" => todo!(),
                    _              => help()
                },
                _                  => help()
            }
            _ => help()
        }
        _ => help()
    } Ok(())
}

fn help() {
    println!("SquirtleCtl available commands:
daemon
  start            : Start SquirtleD
  stop             : Stop SquirtleD

fan-control
  start [OPTS]     : Start Pidgey
    (--standalone) : Start Pidgey without using SquirtleD
                     (only recommended if you're not using any other client)");
}
