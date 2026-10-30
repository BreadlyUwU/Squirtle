mod socket;
use std::{io::{ErrorKind, Read, Write}, os::unix::net::UnixStream, time::Duration};

pub fn start(socket: &str) -> std::io::Result<()> {
    // Program will close early if not executed with root permissions
    squirtle_core::utils::root_check();
    // Now, we must know if a socket has already been created.
    // Because if yes, it can be either a socket from a previous session, or a currently used one.
    old_sock_checker(&socket);
    socket::start(&socket)?;
    Ok(())
}

/// Sniffa the sock & throw it to the trash bin if it stinks
fn old_sock_checker(socket: &str) {
    match std::fs::exists(socket) {
        Ok(true) => {
            match UnixStream::connect(socket) {
                Ok(mut sock) => vibe_check(&mut sock),
                Err(e) => {
                    match e.kind() {
                        ErrorKind::ConnectionRefused => {
                            log::info!("Socket found, but couldn't connect to it {}: {}", socket, e);
                            let _ = match std::fs::remove_file(socket) {
                                Ok(_) => {
                                    log::info!("Old socket cleared.");
                                },
                                Err(e) => {
                                    log::error!("Couldn't delete the old socket file {}: {}", socket, e);
                                    std::process::exit(1)
                                },
                            };
                        },
                        ErrorKind::PermissionDenied => {
                            eprintln!("SquirtleCtl doesn't have permission to read {}, please retry with root permission.", socket);
                            std::process::exit(1)
                        },
                        _ => {
                            eprintln!("Unexpected error while reading {}: {}", socket, e);
                            std::process::exit(2)
                        },
                    }
                },
            };
        },
        Ok(false) => socket::start(socket).unwrap(),
        Err(e) => {
            eprintln!("Cannot check the presence of a currently used or abandoned socket at the expected path: {}", e);
            std::process::exit(1)
        },
    }
}

fn vibe_check(socket: &mut UnixStream) {
    let mut response = String::new();

    socket.set_write_timeout(Some(Duration::from_secs(2))).expect("Cannot set write timeout");
    socket.write_all(b"check").unwrap();
    socket.shutdown(std::net::Shutdown::Write).expect("Cannot shutdown the write stream");

    socket.set_read_timeout(Some(Duration::from_secs(2))).expect("Cannot set read timeout");
    socket.read_to_string(&mut response).unwrap();
    socket.shutdown(std::net::Shutdown::Read).expect("Cannot shutdown the read stream");

    if response == "Ok" {
        println!("SquirtleD is already running!");
        std::process::exit(1);
    } else {
        eprintln!("SquirtleD answered to the vibe-check but with an unexpected response: {}", response);
        std::process::exit(2);
    }
}
