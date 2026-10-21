mod socket;
use std::{io::{ErrorKind, Read, Write}, os::unix::net::UnixStream, time::Duration};

const SOCKET_PATH: &str = "/run/squirtle.sock";

pub fn start() {
    // Program will close early if not executed with root permissions
    squirtle_core::utils::root_check();
    // Now, we must know if a socket has already been created.
    // Because if yes, it can be either a socket from a previous session, or a currently used one.
    old_sock_checker();
}

/// Sniffa the sock & throw it to the trash bin if it stinks
fn old_sock_checker() {
    match std::fs::exists(SOCKET_PATH) {
        Ok(true) => {
            match UnixStream::connect(SOCKET_PATH) {
                Ok(mut sock) => vibecheck(&mut sock),
                Err(e) => {
                    match e.kind() {
                        ErrorKind::ConnectionRefused => {
                            log::info!("Socket found, but couldn't connect to it {}: {}", SOCKET_PATH, e);
                            let _ = match std::fs::remove_file(SOCKET_PATH) {
                                Ok(_) => {
                                    log::info!("Old socket cleared, can now start");
                                    socket::start(SOCKET_PATH)
                                },
                                Err(e) => {
                                    log::error!("Couldn't delete the old socket file {}: {}", SOCKET_PATH, e);
                                    std::process::exit(1)
                                },
                            };
                        },
                        ErrorKind::PermissionDenied => {
                            eprintln!("SquirtleCtl doesn't have permission to read {}, please retry with root permission.", SOCKET_PATH);
                            std::process::exit(1)
                        },
                        _ => {
                            eprintln!("Unexpected error while reading {}: {}", SOCKET_PATH, e);
                            std::process::exit(2)
                        },
                    }

                },
            };
        },
        Ok(false) => socket::start(SOCKET_PATH).unwrap(),
        Err(e) => {
            eprintln!("Cannot check the presence of a currently used or abandoned socket at the expected path: {}", e);
            std::process::exit(1)
        },
    }
}

fn vibecheck(socket: &mut UnixStream) {
    let mut response = String::new();
    socket.set_write_timeout(Some(Duration::from_secs(2))).expect("Cannot set write timeout");
    socket.write_all(b"vibecheck").unwrap();
    socket.shutdown(std::net::Shutdown::Write).expect("Cannot shutdown the write stream");
    socket.set_read_timeout(Some(Duration::from_secs(2))).expect("Cannot set read timeout");
    socket.read_to_string(&mut response).unwrap();
    if response == "Ok" {
	eprintln!("SquirtleD is already running!");
	std::process::exit(1);
    } else {
	eprintln!("SquirtleD answered to the vibe-check but with an unexpected response: {}", response);
	std::process::exit(2);
    }
}

pub fn version() -> String { String::from(env!("CARGO_PKG_VERSION")) }
