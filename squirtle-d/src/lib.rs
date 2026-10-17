use std::{io::{ErrorKind, Read, Write}, os::unix::net::UnixStream, process::exit, time::Duration};

mod socket;
const SOCKET_PATH: &str = "/run/squirtle.sock";

pub fn start() {
    // Before anything we must know of a socket has already been created.
    // Because if yes, it can be either a socket from a previous session, or a currently used one.
    match std::fs::exists(SOCKET_PATH) {
        Ok(true) => {
            match UnixStream::connect(SOCKET_PATH) {
                Ok(mut sock) => {
                    let mut response = String::new();
                    sock.set_write_timeout(Some(Duration::from_secs(2))).expect("Cannot set write timeout");
                    sock.write_all(b"vibecheck").unwrap();
                    sock.shutdown(std::net::Shutdown::Write).expect("Cannot shutdown the write stream");
                    sock.set_read_timeout(Some(Duration::from_secs(2))).expect("Cannot set read timeout");
                    sock.read_to_string(&mut response).unwrap();
                    if response == "Ok" {
                        eprintln!("SquirtleD is already running!");
                        std::process::exit(1);
                    } else {
                        eprintln!("SquirtleD answered to the vibe-check but with an unexpected response: {}", response);
                        std::process::exit(2);
                    }
                },
                Err(e) => {
                    match e.kind() {
                        ErrorKind::ConnectionRefused => {
                            log::info!("Socket found, but couldn't connect to it {}: {}", SOCKET_PATH, e);
                            let _ = match std::fs::remove_file(SOCKET_PATH) {
                                // Old socket cleared, can now start for good
                                Ok(_) => {
                                    log::info!("Old socket cleared, can now start");
                                    socket::start(SOCKET_PATH) 
                                },
                                // Shouldn't happen unless I check for root permission earlier
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
                        _ => todo!(),
                    }

                },
            };
        },
        Ok(false) => socket::start(SOCKET_PATH).unwrap(),
        Err(e) => {
            eprintln!("Cannot check the presence of a currently used or abandonned socket at the expected path: {}", e);
            std::process::exit(1)
        },
    }
}

pub fn version() -> String { String::from(env!("CARGO_PKG_VERSION")) }