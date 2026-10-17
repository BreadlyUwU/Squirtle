// mod config;
use squirtle_d;
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
                "--start"          => squirtle_d::start(),
                "-v" | "--version" => version(),
                _ => help(),
            }
        },
        _ => help()
    } Ok(())
}

fn help() {
    println!("SquirtleCtl:
    <--start> : Start SquirtleD
    <-v|--version> : Show version"
    );
}

#[allow(unused)]
fn not_implemented() { eprintln!("Not implemented yet sowwyyyyy.") }

fn version() {
    println!("🐢 Squirtle Project 💦
SquirtleCtl: Version {}
Squirtle-Core: Version {}
SquirtleD: Version {}
Pidgey: Version {}",
String::from(env!("CARGO_PKG_VERSION")),
    squirtle_core::version(),
    squirtle_d::version(),
    pidgey::version()
    );
}
