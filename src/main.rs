mod config;
use squirtle_d;
use squirtle_core::SYSTEM;
use squirtle_pidgey as pidgey;

fn main() -> std::io::Result<()> {
    if std::env::consts::OS != "linux" {
        eprintln!("This program is designed to be run on Linux.\nOther UNIX-based OS aren't guaranted to be supported in the future.\nSorry for that.\n");
        std::process::exit(1)
    }

    if cfg!(debug_assertions) {
        println!("\n=======================================");
        println!("=              Hello,,,               =");
        println!("= I-I'm s,,, stu,,, stupiiiiid~! >~<  =");
        println!("=======================================\n");
        unsafe { std::env::set_var("RUST_LOG", "debug"); }
    }
    
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        2 => {
            match args[1].as_str() {
                "--start" => squirtle_d::start(), 
                "--demo" => {
                    cpu_demo();
                    // sniffa::gpu::demo();
                },
                "-m" | "--msr" => println!("Not implemented sowwyyyyy."),
                "-f" | "--fan-control" => help(),
                _ => help(),
            }
        },
        3 => {
            match args[1].as_str() {
                "--demo" => match args[2].as_str() {
                    "-c" | "--cpu"     => cpu_demo(),
                    "-g" | "--gpu"     => not_implemented(),
                    _ => help(),
                }
                "-m-" | "--msr" => match args[2] {
                    _ => not_implemented(),
                }
                "-f" | "--fan-control" => match args[2].as_str() {
                    "-d" | "--daemon"     => pidgey::start(pidgey::RunningMode::Real),
                    "-s" | "--simulation" => pidgey::start(pidgey::RunningMode::Simulation),
                    "-t" | "--test"       => pidgey::test(),
                    _            => help(),
                }
                _ => help(),
            };
        }
        _ => help()
    } Ok(())
}

fn cpu_demo() {
    let current_system = SYSTEM.clone();
    current_system.write().unwrap().init_cpuinfo();
    
    let guard = current_system.read().unwrap();
    guard.cpu.as_ref().unwrap().show();
}

fn help() {
    println!("Squirtle
    <--start>\n\t\t- Start SquirtleD
    <-f|--fan-control> (-<d/s/t>|--<daemon|simulation|test>)\n\t\t- Start the built-in fan-control daemon in autonomous mode (require root permissions, unless in simulation mode)
    <-m|--msr> (--read-raw <addr>)\n\t\t- Raw-read a specific MSR address (for testing purposes)
    <--demo> (-<c|g|t>|--<cpu|gpu>)\n\t\t- Run a sniffa CLI demo for debug purposes (without arguments, Squirtle will run all demos)");
}

fn not_implemented() { eprintln!("Not implemented yet sowwyyyyy.") }