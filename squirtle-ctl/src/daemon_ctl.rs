use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::process::Stdio;

// The String the program will have to search for when spawning SquirtleD
const START_MESSAGE: &str = "SquirtleD started";
const ALREADY_RUNNING_MESSAGE: &str = "SquirtleD is already running!";
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub fn start() {
    let bin = find_bin();
    let timer = std::time::Instant::now();
    match std::process::Command::new(bin).env("SQUIRTLE_IS_CHILD", "1").stdout(Stdio::piped()).spawn() {
        Ok(mut process) => {
            let stdout = process.stdout.take().expect("child stdout was not piped");
            let mut buffer = BufReader::new(stdout);

            loop {
                for line in buffer.by_ref().lines() {
                    if timer.elapsed() > TIMEOUT {
                        eprintln!("Error: Timeout");
                        process.kill().unwrap();
                        std::process::exit(1)
                    }
                    let line = line.unwrap();
                    match line.as_str() {
                        START_MESSAGE           => return,
                        ALREADY_RUNNING_MESSAGE => { println!("{ALREADY_RUNNING_MESSAGE}"); return },
                        _                       => continue
                    }

                }

                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        },
        Err(e) => {
            eprintln!("Failed to start squirtled: {}", e);
            std::process::exit(1);
        }
    };
}

pub fn stop() {
    if let Err(e) = message("stop") {
        eprintln!("{e}");
    }
}

fn find_bin() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from("./target/debug/squirtled")
    } else {
        let path = std::env::var_os("PATH").unwrap();
        let find = std::env::split_paths(&path)
            .map(|directory| directory.join("squirtled"))
            .find(|findings| {
                findings.is_file()
            });

        if let Some(path) = find { path }
        else {
            eprintln!("Error: SquirtleD doesn't seems to be correctly installed.");
            eprintln!("\"squirtled\" hasn't been found in the PATH");
            std::process::exit(1);
        }
    }
}

fn message(msg: &str) -> std::io::Result<()> {
    let socket: &str = "/run/squirtled.sock";
    match UnixStream::connect(socket) {
        Ok(mut sock) => {
            sock.write_all(msg.as_bytes())?;
            sock.shutdown(std::net::Shutdown::Both)?;
            Ok(())
        },
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => eprintln!("No socket found at {}. SquirtleD has probably not been started, or has been but with the socket set to an another location than the default one.", socket),
                ErrorKind::ConnectionRefused => eprintln!("Socket found, but couldn't connect to it {}: {}", socket, e),
                ErrorKind::PermissionDenied => eprintln!("You do not have permission to interract with SquirtleD."),
                _ => eprintln!("Unexpected error while reading {}: {}", socket, e),
            } Err(e)
        },
    }
}
